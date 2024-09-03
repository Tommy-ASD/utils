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
    coordinates.iter().map(|c| map_to_vec(c)).collect()
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
    c.as_array()
        .ok_or_else(|| {
            traceback!("Expected an array as a parameter").with_extra_data(json!({ "c": c }))
        })
        .and_then(|a| a.iter().map(map_to_vec_inner).collect())
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
