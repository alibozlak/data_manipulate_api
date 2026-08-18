# data_manipulate_api

An HTTP service that rescales a training data set so every value falls into a
single-digit range, and reports the power of ten each column was divided by.

Feeding a regression model raw features that span wildly different magnitudes
(a price in the tens of thousands next to a room count in the single digits)
makes gradient descent converge slowly or not at all. This service performs that
rescaling step over HTTP, returning both the scaled data and the exponents, so a
caller can map the resulting coefficients back to the original units.

Built with [axum](https://github.com/tokio-rs/axum) on tokio.

## API

### `POST /manipulate-datas`

Accepts `multipart/form-data` with a single field named `dataset` holding the
JSON payload as text. The request body is capped at 32 MiB.

**Request payload**

```json
{
  "inputs": [[116.6, 5.0, 13.0, 3.0], [655.0, 1.0, 22.0, 2.5]],
  "outputs": [2300000.0, 1750000.0],
  "initial_coefficients": [0.0, 0.0, 0.0, 0.0, 0.0]
}
```

| Field | Type | Meaning |
| --- | --- | --- |
| `inputs` | `[[float]]` | One entry per sample, each holding that sample's `n` feature values. |
| `outputs` | `[float]` | The expected output of the sample at the same index in `inputs`. |
| `initial_coefficients` | `[float]` | Starting coefficients: one per feature plus the bias, so `n + 1` values. |

**Response payload**

```json
{
  "ratios": [2, 0, 1, 0, 6],
  "inputs": [[1.166, 5.0, 1.3, 3.0], [6.55, 1.0, 2.2, 2.5]],
  "outputs": [2.3, 1.75],
  "initial_coefficients": [0.0, 0.0, 0.0, 0.0, 0.0]
}
```

`ratios` holds `n + 1` exponents — one per feature column, with the exponent for
`outputs` in the last slot. A value `r` means the column was divided by `10^r`.
`initial_coefficients` is echoed back unchanged.

**Scaling rule.** For a value `v`, the exponent is `len(digits before the
decimal point of |v|) - 1`, and the scaled value is `v * 10^-r`. So `116.6`
becomes `1.166` with exponent `2`, while `5.0` is left alone with exponent `0`.

> **Note:** the exponent is computed and applied per value, but only the first
> sample's exponents end up in `ratios`. If a later sample has a different
> magnitude in some column, that column is scaled inconsistently and `ratios`
> will not describe it. Keep magnitudes uniform within a column, or treat
> `ratios` as descriptive of row 0 only.

**Errors** — all returned as plain text:

| Status | Cause |
| --- | --- |
| `400` | No `dataset` field in the form, or the field could not be read. |
| `400` | The payload is not valid JSON or does not match the shape above. |
| `400` | `inputs` is empty. |
| `400` | `inputs` and `outputs` have different lengths. |
| `400` | Samples in `inputs` do not all have the same feature count. |
| `400` | `len(initial_coefficients) != n + 1`. |
| `500` | The result could not be serialised back to JSON. |

## Running with Docker

```bash
docker build -t data_manipulate_api .
```

```bash
docker run --rm -p 3001:3001 data_manipulate_api
```

The service is then reachable at `http://localhost:3001`.

The image is a two-stage build: `rust:1.95-slim-bookworm` compiles the release
binary, and only that binary is copied into a `debian:bookworm-slim` runtime
that runs as a non-root `app` user. Dependencies are compiled in their own layer,
so editing `src/` does not trigger a full rebuild of the dependency tree.

### Configuration

| Variable | Default |
| --- | --- | 
| `BIND_ADDR` | `127.0.0.1:3001` |

To serve on a different port, change both the bind address and the mapping:

```bash
docker run --rm -e BIND_ADDR=0.0.0.0:8080 -p 8080:8080 data_manipulate_api
```

## Running locally

Requires Rust 1.85 or newer — the crate is on edition 2024.

```bash
cargo run --release
```

This listens on `127.0.0.1:3001`. Set `BIND_ADDR` to change it.

## Example request

With a `dataset.json` file holding the payload:

```bash
curl -X POST http://localhost:3001/manipulate-datas -F "dataset=@dataset.json"
```

Or inline:

```bash
curl -X POST http://localhost:3001/manipulate-datas -F 'dataset={"inputs":[[116.6,5.0]],"outputs":[2300000.0],"initial_coefficients":[0.0,0.0,0.0]}'
```

## Project layout

| File | Responsibility |
| --- | --- |
| [`src/main.rs`](src/main.rs) | Router setup, body limit, and the tokio listener. |
| [`src/request_with_json_file.rs`](src/request_with_json_file.rs) | The handler: pulls `dataset` out of the multipart form and maps failures to status codes. |
| [`src/json_converter.rs`](src/json_converter.rs) | Payload types, validation, and the `JsonConverterError` enum. |
| [`src/data_manipulate.rs`](src/data_manipulate.rs) | The rescaling itself. |
