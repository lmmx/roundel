// src/data/api.rs
use crate::app::simulation::VehicleType;
use crate::utils::log::{self, LogCategory};
use crate::tb8::{Prediction, Response as Tb8Response};
use wasm_bindgen::JsCast;
use web_sys::Response;

/// Fetch real-time arrival data for a specific line
pub async fn fetch_arrivals_for_line(line_id: &str) -> Result<Vec<Prediction>, String> {
    log::info_with_category(
        LogCategory::Simulation,
        &format!("Fetching arrivals for line: {}", line_id),
    );

    // Construct the API URL
    let api_url = format!("https://tb8-rs-production.up.railway.app/arrivals-by-lines?query={}", line_id);

    // Create a future to fetch the data
    let window = web_sys::window().ok_or("No window object available")?;
    let promise = window.fetch_with_str(&api_url);

    // Convert the Promise<Response> to a Future<Result<Response, JsValue>>
    let response_future = wasm_bindgen_futures::JsFuture::from(promise);

    // Await the response
    let response_value = match response_future.await {
        Ok(val) => val,
        Err(e) => return Err(format!("Failed to fetch arrivals: {:?}", e)),
    };

    let response: Response = response_value
        .dyn_into()
        .map_err(|_| "Failed to convert response".to_string())?;

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    // Get the response text
    let text_promise = response
        .text()
        .map_err(|e| format!("Failed to get response text: {:?}", e))?;
    let text_future = wasm_bindgen_futures::JsFuture::from(text_promise);

    let text = match text_future.await {
        Ok(val) => val.as_string().ok_or("Response is not a string")?,
        Err(e) => return Err(format!("Failed to get response text: {:?}", e)),
    };

    // Parse the JSON using your existing Prediction model
    let response: Tb8Response<Prediction> =
        match serde_json::from_str(&text) {
            Ok(resp) => resp,
            Err(e) => {
                let error_msg = format!("Failed to parse arrivals JSON: {}", e);
                log::error_with_category(LogCategory::Simulation, &error_msg);
                return Err(error_msg);
            }
        };

    // Check if the response was successful
    if !response.success {
        return Err("Arrivals response was unsuccessful".to_string());
    }

    log::info_with_category(
        LogCategory::Simulation,
        &format!("Successfully loaded {} arrivals for {}", response.results.len(), line_id),
    );

    Ok(response.results)
}

/// Determine vehicle type from line ID
pub fn get_vehicle_type_for_line(line_id: &str) -> VehicleType {
    match line_id {
        "dlr" => VehicleType::Train,
        "elizabeth" => VehicleType::Train,
        "london-cable-car" => VehicleType::Train,
        "waterloo-city" | "victoria" | "piccadilly" | "northern" | "metropolitan"
        | "jubilee" | "hammersmith-city" | "district" | "circle" | "central"
        | "bakerloo" => VehicleType::Train,
        "thameslink" | "tram" => VehicleType::Train,
        "liberty" | "lioness" | "mildmay" | "suffragette" | "weaver" | "windrush" => VehicleType::Train,
        _ => VehicleType::Bus, // Default to bus for other IDs
    }
}
