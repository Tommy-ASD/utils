use serde_json::{json, Value};

use traceback_error::{traceback, TracebackError};
/*
This function takes in a set of coordinates in the following format:
[
    [
        [ x, y ],
        [ x, y ],
        [ x, y ],
        [ x, y ],
        [ x, y ]
    ]
]
*/
// Converts a vector of JSON `Value` objects representing coordinates to a nested vector of floating-point numbers.
// Returns a `Result` containing the nested vector if the conversion is successful,
// or an error message as a `String` if there's a failure during the conversion process.
pub fn coords_to_vec(coordinates: &Vec<Value>) -> Result<Vec<Vec<Vec<f64>>>, TracebackError> {
    coordinates.iter().map(|c| map_to_vec(c)).collect()
}

// Helper function that converts a JSON `Value` object to a vector of vectors of floating-point numbers.
// This function is used internally by `coords_to_vec`.
// Returns a `Result` containing the vector of vectors if the conversion is successful,
// or an error message as a `String` if the input is not an array.
fn map_to_vec(c: &Value) -> Result<Vec<Vec<f64>>, TracebackError> {
    c.as_array()
        .ok_or_else(|| {
            traceback!("Expected an array as a parameter").with_extra_data(json!({ "c": c }))
        })
        .and_then(|a| a.iter().map(map_to_vec_inner).collect())
}

// Helper function that converts a JSON `Value` object to a vector of floating-point numbers.
// This function is used internally by `map_to_vec`.
// Returns a `Result` containing the vector of floating-point numbers if the conversion is successful,
// or an error message as a `String` if the input is not an array or if the values cannot be parsed as floating-point numbers.
fn map_to_vec_inner(c: &Value) -> Result<Vec<f64>, TracebackError> {
    c.as_array()
        .ok_or_else(|| {
            traceback!("Expected an array as a parameter").with_extra_data(json!({ "c": c }))
        })
        .map(|a| a.iter().map(|c| c.as_f64().unwrap_or(0.0)).collect())
}
