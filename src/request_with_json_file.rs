use axum::extract::Multipart;
use axum::http::StatusCode;
use crate::{data_manipulate, json_converter};

pub async fn convert_to_json_string(
    mut multipart: Multipart
) -> Result<String, (StatusCode, String)> {
    let mut payload: Option<String> = None;

    while let Some(field) = multipart.next_field().await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
    {
        if field.name() == Some("dataset") {
            payload = Some(field.text().await
                .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?);
        }
    }

    let mut json_data : String =  payload.ok_or((
        StatusCode::BAD_REQUEST,
        "Form hasn't a 'dataset' named field !!".to_string(),
    ))?;

    let (inputs, outputs, initial_coefficients)
        = json_converter::manipule_from_json(&json_data)
        .map_err(|error| (StatusCode::BAD_REQUEST, error.to_string()))?;

    let (result_inputs, result_outputs, ratios)
        = data_manipulate::manipulate_datas_between_0_and_10(inputs, outputs);

    json_data = json_converter::coefficients_to_json(ratios, result_inputs, result_outputs, initial_coefficients)
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;

    Ok(json_data)
}