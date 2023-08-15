use std::{
    fs::{create_dir_all, read_to_string, File},
    io::Write,
};

use serde_json::{Map, Value};

/*
This function is used to split the roller.json file into multiple files.
The reason for this is that the file is too large to be uploaded to Surreal.
It should work with any JSON array, but it's only been tested with roller.json.

Parameters:
    filepath: &str - the path to the JSON file
    split_size: usize - the size of the split files

Returns:
    None, as it only writes to files

Possible problems:
    - if the file can not be read, the function will exit early
    - if the file is malformed, the function will exit early
    - if the JSON file is not an array, the function will exit early
    - if writing to the files fails, it will not work
    - if the file is too large and the host machine doesn't have enough memory,
      i am not sure what will happen (probably a panic)
    - if the file is too large and the host machine doesn't have enough disk space,
      i am also not sure what will happen (probably a panic)

Possible improvements:
    - reduce code repetition
    - general code cleanup

Calls:
    - none
*/
pub fn split_array_from_json_file(filepath: &str, split_size: usize) {
    let str = match read_to_string(filepath) {
        Ok(s) => s,
        Err(e) => {
            println!("Error when reading roller JSON: {}", e);
            return;
        }
    };
    let parsed: serde_json::Value = match serde_json::from_str(&str) {
        Ok(p) => p,
        Err(e) => {
            println!("Error when parsing roller JSON: {}", e);
            return;
        }
    };
    let parsed = match parsed.as_array() {
        Some(p) => p,
        None => {
            println!("Error when parsing roller JSON: not an array");
            return;
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
            println!("Error when creating directory: {}", e);
            return;
        }
    };
    let mut i = 0;
    let parsed_split = parsed.chunks(split_size);
    for chunk in parsed_split {
        let mut file = match File::create(format!(".{folder_path}/{i}.{extension}")) {
            Ok(f) => f,
            Err(e) => {
                println!("Error when creating file: {}", e);
                return;
            }
        };
        let chunk = match serde_json::to_string(chunk) {
            Ok(c) => c,
            Err(e) => {
                println!("Error when parsing chunk: {}", e);
                return;
            }
        };
        match file.write_all(chunk.as_bytes()) {
            Ok(_) => {}
            Err(e) => {
                println!("Error when writing to file: {}", e);
                return;
            }
        };
        i += 1;
    }
}

#[macro_export]
macro_rules! extract_nested_json {
    ($json:expr, $ret_type:ty, $key:expr) => {
        || {
            let j = $json[$key].clone();
            let parsed_to_type: $ret_type = match serde_json::from_value(j) {
                Ok(v) => v,
                Err(e) => return Err(
                    format!(
                        "Error when getting key {key} from json {json} with expected type {type}: {error}", key = $key, json = $json, type = stringify!($ret_type), error = e
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
