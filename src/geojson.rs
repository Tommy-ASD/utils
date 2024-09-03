use traceback_error::{
    serde_json::{json, Value},
    traceback, TracebackError,
};

/// Converts a vector of JSON `Value` objects representing coordinates to a nested vector of floating-point numbers.
/// # Arguments
/// * `coordinates` - A reference to a vector of JSON `Value` objects representing coordinates.
///
/// # Returns
/// * `Result<Vec<Vec<Vec<f64>>>, TracebackError>` - A `Result` containing the nested vector if the conversion is successful,
/// or an error message as a `TracebackError` if there's a failure during the conversion process.
///
/// # Example
/// ```
/// use serde_json::{json, Value};
/// use traceback_error::TracebackError;
/// use utils::geojson::coords_to_vec;
///
/// fn main() {
///     let coordinates = vec![
///         json!([[1.0, 2.0], [3.0, 4.0], [5.0, 6.0]]),
///         json!([[7.0, 8.0], [9.0, 10.0], [11.0, 12.0]]),
///     ];
///
///     let result = coords_to_vec(&coordinates);
///
///     match result {
///         Ok(nested_vec) => {
///             println!("{:?}", nested_vec);
///         }
///         Err(err) => {
///             eprintln!("Error: {:?}", err);
///         }
///     }
/// }
/// ```
pub fn coords_to_vec(coordinates: &Vec<Value>) -> Result<Vec<Vec<Vec<f64>>>, TracebackError> {
    let mut result = vec![];
    for coordinate in coordinates {
        match map_to_vec(coordinate) {
            Ok(val) => result.push(val),
            Err(e) => return Err(traceback!(err e).with_extra_data(json!({"coords": coordinates})))
        }
    }
    Ok(result)
}

/// Helper function that converts a JSON `Value` object to a vector of vectors of floating-point numbers.
/// This function is used internally by `coords_to_vec`.
///
/// # Arguments
/// * `c` - A reference to a JSON `Value` object to be converted.
///
/// # Returns
/// * `Result<Vec<Vec<f64>>, TracebackError>` - A `Result` containing the vector of vectors if the conversion is successful,
/// or an error message as a `TracebackError` if the input is not an array.
fn map_to_vec(c: &Value) -> Result<Vec<Vec<f64>>, TracebackError> {
    let c_array = match c.as_array() {
        Some(ok) => ok,
        None => return Err(traceback!("Expected an array as a parameter").with_extra_data(json!({ "c": c })))
    };
    let mut result = vec![];
    for element in c_array {
        match map_to_vec_inner(element) {
            Ok(v) => result.push(v),
            Err(e) => return Err(traceback!(err e).with_extra_data(json!({
                "c_array": c_array
            })))
        }
    }
    Ok(result)
}

/// Helper function that converts a JSON `Value` object to a vector of floating-point numbers.
/// This function is used internally by `map_to_vec`.
///
/// # Arguments
/// * `value` - A reference to a JSON `Value` object to be converted.
///
/// # Returns
/// * `Result<Vec<f64>, TracebackError>` - A `Result` containing the vector of floating-point numbers if the conversion is successful,
/// or an error message as a `TracebackError` if the input is not an array or if the values cannot be parsed as floating-point numbers.
fn map_to_vec_inner(value: &Value) -> Result<Vec<f64>, TracebackError> {
    value.as_array()
        .ok_or_else(|| {
            traceback!("Expected an array as a parameter").with_extra_data(json!({ "value": value }))
        })
        .map(|value_element| {
            value_element.iter()
                // enumerate really isn't necessary here
                // but debugging is a lot easier if we know the index where the error happened
                .enumerate()
                .map(|(i, element_inner)| {
                    element_inner.as_f64().unwrap_or_else(|| {
                        traceback!(format!("Failed to parse index {i} into f64 in value"))
                            .with_extra_data(json!({
                                "value": value,
                                "value_element": value_element,
                                "index": i,
                                "element_inner": element_inner
                            }));
                        0.0
                    })
                })
                .collect()
        })
}
