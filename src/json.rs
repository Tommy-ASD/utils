use std::{
    fs::{create_dir_all, read_to_string, File},
    io::Write,
};

use serde_json::{Map, Value};

use traceback_error::{traceback, TracebackError};

/// Splits a JSON array from a file into multiple smaller files.
///
/// The purpose of this function is to split a large JSON array stored in a file
/// into smaller files to make it manageable for upload or processing.
///
/// # Arguments
///
/// * `filepath` - A string representing the path to the input JSON file.
/// * `split_size` - The size of each split file.
///
/// # Returns
///
/// Returns `Ok(())` if the splitting process is successful, or `Err(TracebackError)` on failure.
///
/// # Possible Problems
///
/// This function can encounter the following issues:
///
/// - If the file cannot be read, the function will return an error.
/// - If the file is malformed JSON, the function will return an error.
/// - If the JSON file is not an array, the function will return an error.
/// - If writing to the split files fails, it will result in an error.
/// - If the file is too large and the host machine doesn't have enough memory,
///   it may lead to a panic.
/// - If the file is too large and the host machine doesn't have enough disk space,
///   it may also lead to a panic.
///
/// # Possible Improvements
///
/// Some possible improvements for this function include:
///
/// - Reducing code repetition to improve maintainability.
/// - General code cleanup and optimization.
///
/// # Example
///
/// ```rust
/// use std::collections::HashMap;
/// use serde::Deserialize;
/// use traceback_error::TracebackError;
/// use your_module_name::split_array_from_json_file;
///
/// #[derive(Debug, Deserialize)]
/// struct Item {
///     // Define your struct fields here.
/// }
///
/// #[tokio::main]
/// async fn main() -> Result<(), TracebackError> {
///     let file_path = "path/to/your/json_file.json";
///     let split_size = 100; // Specify the desired split size.
///
///     let result = split_array_from_json_file(file_path, split_size);
///
///     match result {
///         Ok(_) => {
///             println!("JSON array successfully split.");
///         }
///         Err(err) => {
///             eprintln!("Error: {:?}", err);
///         }
///     }
///
///     Ok(())
/// }
/// ```
///
/// In this example, the `split_array_from_json_file` function is used to split a JSON array from a file into smaller files.
/// Make sure to specify the correct file path and desired split size for your use case.
pub fn split_array_from_json_file(filepath: &str, split_size: usize) -> Result<(), TracebackError> {
    let str = match read_to_string(filepath) {
        Ok(s) => s,
        Err(e) => {
            return Err(traceback!(e, "Error when reading roller JSON"));
        }
    };
    let parsed: serde_json::Value = match serde_json::from_str(&str) {
        Ok(p) => p,
        Err(e) => {
            return Err(traceback!(e, "Error when parsing roller JSON"));
        }
    };
    let parsed = match parsed.as_array() {
        Some(p) => p,
        None => {
            return Err(traceback!("Error when parsing roller JSON: not an array"));
        }
    };
    let folder_path = filepath.split(".").collect::<Vec<&str>>()[filepath.split(".").count() - 2];
    let extension = filepath.split(".").collect::<Vec<&str>>()[filepath.split(".").count() - 1];
    let filename = filepath.split("/").collect::<Vec<&str>>()[filepath.split("/").count() - 1]
        .split(".")
        .collect::<Vec<&str>>()[0];
    println!("Path: .{folder_path}/{filename}/0.{extension}");
    println!("Folder path: {folder_path}");
    println!("Extension: {extension}");
    println!("Filename: {filename}");
    match create_dir_all(format!(".{folder_path}")) {
        Ok(_) => {}
        Err(e) => {
            return Err(traceback!(e, "Error when creating directory"));
        }
    };
    let mut i = 0;
    let parsed_split = parsed.chunks(split_size);
    for chunk in parsed_split {
        let mut file = match File::create(format!(".{folder_path}/{i}.{extension}")) {
            Ok(f) => f,
            Err(e) => {
                return Err(traceback!(e, "Error when creating file"));
            }
        };
        let chunk = match serde_json::to_string(chunk) {
            Ok(c) => c,
            Err(e) => {
                return Err(traceback!(e, "Error when parsing chunk"));
            }
        };
        match file.write_all(chunk.as_bytes()) {
            Ok(_) => {}
            Err(e) => {
                return Err(traceback!(e, "Error when writing to file"));
            }
        };
        i += 1;
    }
    Ok(())
}

#[macro_export]
macro_rules! extract_nested_json {
    ($json:expr, $ret_type:ty, $key:expr) => {
        || {
            let j = $json[$key].clone();
            let parsed_to_type: $ret_type = match serde_json::from_value(j) {
                Ok(v) => v,
                Err(e) => return Err(
                    traceback!(e,
                        format!(
                            "Error when getting key {key} from json {json} with expected type {type}",
                            key = $key,
                            json = $json,
                            type = stringify!($ret_type),
                        )
                    )
                ),
            };
            Ok(parsed_to_type)
        }
    };
    ($json:expr, $ret_type:ty, $key:expr, $($keys:expr),*) => {
        extract_nested_json!($json[$key], $ret_type, $($keys),*)
    };
}

/*
This function is used to detect nested json values.
It takes a json object, and returns the keys of all json values that are nested in the object.
It does this recursively, so if there are nested objects in the nested objects, it will return the keys of those as well.
This is returned as a vector of strings.
*/
pub fn detect_nested_json(json: &Map<String, Value>) -> Vec<String> {
    let mut keys = Vec::new();
    for (key, value) in json {
        match value {
            Value::Object(nested_obj) => {
                if !nested_obj.is_empty() {
                    let nested_keys = detect_nested_json(nested_obj);
                    for nested_key in nested_keys {
                        keys.push(format!("{}.{}", key, nested_key));
                    }
                }
            }
            _ => {
                keys.push(key.to_string());
            }
        }
    }
    keys
}

/// Generates a JSON schema from a given JSON-like data structure.
///
/// This function takes a reference to a JSON-like data structure represented by
/// the `serde_json::Value` enum and generates a JSON schema describing its structure.
/// The schema describes the types of values, arrays, and objects contained in the input
/// data, and their hierarchical relationships.
///
/// # Arguments
///
/// * `input`: A reference to the JSON-like data structure (`serde_json::Value`) for which
///   you want to generate a schema.
///
/// # Returns
///
/// A `serde_json::Value` representing the JSON schema.
///
/// # Example
///
/// ```rust
/// use serde_json::json;
///
/// let input_data = json!({
///     "name": "John",
///     "age": 30,
///     "is_student": false,
///     "hobbies": ["reading", "swimming"],
/// });
///
/// let schema = generate_schema(&input_data);
///
/// // The `schema` now contains a JSON schema describing the structure of `input_data`.
/// ```
///
/// # Note
///
/// This function recursively traverses the input data structure to generate the schema.
/// It supports various data types including null, boolean, number, string, arrays, and objects.
/// The generated schema is represented as a `serde_json::Value`.
pub fn generate_schema(input: &serde_json::Value) -> serde_json::Value {
    // Match the input value to determine its type and generate the schema accordingly
    match input {
        serde_json::Value::Null => serde_json::json!({"type": "null"}),
        serde_json::Value::Bool(_) => serde_json::json!({"type": "boolean"}),
        serde_json::Value::Number(_) => serde_json::json!({"type": "number"}),
        serde_json::Value::String(_) => serde_json::json!({"type": "string"}),
        serde_json::Value::Array(arr) => {
            // Generate the schema for array values
            let items_schema = arr.iter().fold(None, |schema, item| {
                let item_schema = generate_schema(item);
                match schema {
                    Some(schema) => {
                        if schema != item_schema {
                            Some(serde_json::json!([schema, item_schema]))
                        } else {
                            Some(schema)
                        }
                    }
                    None => Some(item_schema),
                }
            });

            serde_json::json!({
                "type": "array",
                "items": items_schema.unwrap_or(serde_json::json!({}))
            })
        }
        serde_json::Value::Object(obj) => {
            // Generate the schema for object values
            let properties: serde_json::Map<String, serde_json::Value> = obj
                .iter()
                .map(|(key, value)| {
                    let prop_schema = generate_schema(value);
                    (key.clone(), prop_schema)
                })
                .collect();

            serde_json::json!({
                "type": "object",
                "properties": properties,
                "required": obj.keys().cloned().collect::<Vec<String>>()
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_generate_schema_null() {
        let input = Value::Null;
        let expected_schema = json!({"type": "null"});
        assert_eq!(generate_schema(&input), expected_schema);
    }

    #[test]
    fn test_generate_schema_boolean() {
        let input = Value::Bool(true);
        let expected_schema = json!({"type": "boolean"});
        assert_eq!(generate_schema(&input), expected_schema);
    }

    #[test]
    fn test_generate_schema_number() {
        let input = Value::Number(serde_json::Number::from(42));
        let expected_schema = json!({"type": "number"});
        assert_eq!(generate_schema(&input), expected_schema);
    }

    #[test]
    fn test_generate_schema_string() {
        let input = Value::String("Hello, World!".to_string());
        let expected_schema = json!({"type": "string"});
        assert_eq!(generate_schema(&input), expected_schema);
    }

    #[test]
    fn test_generate_schema_array() {
        let input = json!([1, 2, 3]);
        let expected_schema = json!({
            "type": "array",
            "items": {"type": "number"}
        });
        assert_eq!(generate_schema(&input), expected_schema);
    }

    #[test]
    fn test_generate_schema_object() {
        let input = json!({"key1": 42, "key2": "value"});
        let expected_schema = json!({
            "type": "object",
            "properties": {
                "key1": {"type": "number"},
                "key2": {"type": "string"}
            },
            "required": ["key1", "key2"]
        });
        assert_eq!(generate_schema(&input), expected_schema);
    }
}
