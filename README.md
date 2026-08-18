# data_manipulate_api

An HTTP service that divides each column of a training data set by a power of
ten, and reports the exponent it used.

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
  "outputs": [2300000.0, 1750000.0]
}
```

| Field | Type | Meaning |
| --- | --- | --- |
| `inputs` | `[[float]]` | One entry per sample, each holding that sample's `n` feature values. |
| `outputs` | `[float]` | The expected output of the sample at the same index in `inputs`. |

**Response payload**

```json
{
  "ratios": [2, 0, 1, 0, 6],
  "inputs": [[1.166, 5.0, 1.3, 3.0], [6.55, 1.0, 2.2, 2.5]],
  "outputs": [2.3, 1.75]
}
```

`ratios` holds `n + 1` exponents — one per feature column, with the exponent for
`outputs` in the last slot. A value `r` means the column was divided by `10^r`.

**Scaling rule.** A column's exponent is taken from its **first sample**:
`r = len(digits before the decimal point of |v0|) - 1`, where `v0` is that
column's value in `inputs[0]`. Every value in the column is then scaled to
`v * 10^-r`, the first one included. `outputs` gets its own exponent the same
way, from `outputs[0]`, and that one occupies the last slot of `ratios`.

So in the example above the first feature column takes `r = 2` from `116.6`, and
`655.0` is divided by that same `10^2` to land on `6.55`. A column whose first
value is already below 10 keeps `r = 0` and is left alone.

Because one exponent covers a whole column, the transform is a plain linear
rescaling. That is what makes `ratios` enough to map a model's coefficients back
to the original units:

```text
a_j = a'_j * 10^(r_y - r_j)      b = b' * 10^r_y
```

where `a'` and `b'` are the coefficients trained on the scaled data, `r_j` is
that feature's exponent, and `r_y` is the last entry of `ratios`.

> **Note:** the exponent comes from the first sample rather than from the
> column's largest value, so anything bigger than the first sample is not
> brought under 10. A column of `[55.0, 165.0]` takes `r = 1` from `55.0` and
> comes back as `[5.5, 16.5]`. The scaling is still uniform across the column
> and the mapping back is still exact — it is only the "single digit" part that
> the first row cannot promise on behalf of the rest.
>
> It cuts the other way too. A first sample smaller than the rest leaves the
> column barely scaled: `[5.0, 900000.0]` takes `r = 0` and passes through
> untouched, while `[900000.0, 5.0]` — the same two values, reordered — takes
> `r = 5` and becomes `[9.0, 0.00005]`. Ordering the data so the first sample
> carries each column's largest magnitude gets the most conditioning out of this
> service, and keeps the scaled values inside a single digit.

**Errors** — all returned as plain text:

| Status | Cause |
| --- | --- |
| `400` | No `dataset` field in the form, or the field could not be read. |
| `400` | The payload is not valid JSON or does not match the shape above. |
| `400` | `inputs` is empty. |
| `400` | `inputs` and `outputs` have different lengths. |
| `400` | Samples in `inputs` do not all have the same feature count. |
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

| Variable | Default | Notes |
| --- | --- | --- |
| `BIND_ADDR` | `127.0.0.1:3001` | The image overrides this to `0.0.0.0:3001`. A container bound to loopback is unreachable from its own host and from every other container, publishing a port does not change that. |

Keeping the service private is the network's job, not the bind address's: publish
nothing, or publish to loopback on the host side.

```bash
docker run --rm -e BIND_ADDR=0.0.0.0:8080 -p 127.0.0.1:8080:8080 data_manipulate_api
```

The compose stack that runs the chain — kept outside these repositories, pulling its
images from Docker Hub — gives this service no `ports:` entry at all, so the only
thing that reaches it is `learn_model_with_linear_regression_api` on the same private
network.

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
curl -X POST http://localhost:3001/manipulate-datas -F 'dataset={"inputs":[[116.6,5.0]],"outputs":[2300000.0]}'
```

## Project layout

| File | Responsibility |
| --- | --- |
| [`src/main.rs`](src/main.rs) | Router setup, body limit, and the tokio listener. |
| [`src/request_with_json_file.rs`](src/request_with_json_file.rs) | The handler: pulls `dataset` out of the multipart form and maps failures to status codes. |
| [`src/json_converter.rs`](src/json_converter.rs) | Payload types, validation, and the `JsonConverterError` enum. |
| [`src/data_manipulate.rs`](src/data_manipulate.rs) | The rescaling itself. |
