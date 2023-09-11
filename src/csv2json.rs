use csv::Reader;
use serde_json::{json, Value};

use traceback_error::{traceback, TracebackError};

/// Converts a CSV data represented by a `csv::Reader<&[u8]>` into a `serde_json::Value`.
///
/// ## Arguments
///
/// * `csv` - A `csv::Reader<&[u8]>` containing the CSV data to be converted.
///
/// ## Returns
///
/// * `Result<serde_json::Value, TracebackError>` - A `Result` containing the resulting JSON data as `serde_json::Value`
///   if the conversion is successful, or an error message as a `TracebackError` if there's an issue during the conversion process.
///
/// ## Assumptions
///
/// - The first row of the CSV is considered the header row.
/// - All other rows are treated as data rows.
/// - All data in the CSV is assumed to be in string format.
///
/// ## Notes
///
/// - Some data may be lost during the conversion because serde_json automatically sorts CSV headers alphabetically.
///
/// ## Example
///
/// ```rust
/// use csv::Reader;
/// use serde_json::Value;
/// use traceback_error::TracebackError;
/// use utils::csv2json::csv_to_json;
///
/// fn main() {
///     // Create a CSV reader from a byte slice (replace with your actual CSV data)
///     let csv_data: &[u8] = b"Name,Age,Location\nAlice,25,New York\nBob,30,Los Angeles";
///     let reader = Reader::from_reader(csv_data);
///
///     // Convert the CSV data to JSON
///     let result = csv_to_json(reader);
///
///     match result {
///         Ok(json_data) => {
///             println!("{}", json_data);
///         }
///         Err(err) => {
///             eprintln!("Error: {:?}", err);
///         }
///     }
/// }
/// ```
///
/// In this example, `csv_data` is a byte slice representing CSV data. The function `csv_to_json` is used to convert the CSV data into JSON format.
/// The resulting JSON data can be used as needed.
pub fn csv_to_json<T: std::io::Read>(
    mut csv: Reader<T>,
) -> Result<serde_json::Value, TracebackError> {
    let headers = match csv.headers().cloned() {
        Ok(headers) => headers,
        Err(e) => {
            return Err(traceback!("Failed to read CSV headers")
                .with_extra_data(json!({ "error": e.to_string() })))
        }
    };
    let mut records = Vec::new();
    for result in csv.records() {
        let record = match result {
            Ok(record) => record,
            Err(e) => {
                return Err(traceback!("Failed to read CSV record")
                    .with_extra_data(json!({ "error": e.to_string() })))
            }
        };
        let mut obj = serde_json::Map::new();
        for (i, header) in headers.iter().enumerate() {
            let current_rec = match record.get(i) {
                Some(current_rec) => current_rec,
                None => {
                    return Err(traceback!("Failed to get current record")
                        .with_extra_data(json!({ "record": format!("{:?}", record) })))
                }
            };
            obj.insert(
                header.to_string(),
                serde_json::Value::String(current_rec.to_string()),
            );
        }
        records.push(serde_json::Value::Object(obj));
    }
    Ok(serde_json::Value::Array(records))
}

/// Converts a `serde_json::Value` into a CSV-formatted string.
///
/// ## Arguments
///
/// * `json` - A `serde_json::Value` containing the JSON data to be converted to CSV.
///
/// ## Returns
///
/// * `Result<String, TracebackError>` - A `Result` containing the CSV-formatted string if the conversion is successful,
///   or an error message as a `TracebackError` if there's an issue during the conversion process.
///
/// ## Example
///
/// ```rust
/// use serde_json::{Value, json};
/// use traceback_error::TracebackError;
/// use utils::csv2json::json_to_csv;
///
/// fn main() {
///     // Create a JSON object (replace with your actual JSON data)
///     let json_data = json!([
///         {"Name": "Alice", "Age": 25, "Location": "New York"},
///         {"Name": "Bob", "Age": 30, "Location": "Los Angeles"}
///     ]);
///
///     // Convert the JSON data to CSV
///     let result = json_to_csv(json_data);
///
///     match result {
///         Ok(csv_string) => {
///             println!("{}", csv_string);
///         }
///         Err(err) => {
///             eprintln!("Error: {:?}", err);
///         }
///     }
/// }
/// ```
///
/// In this example, `json_data` is a JSON object containing an array of records. The function `json_to_csv` is used to convert the JSON data into a CSV-formatted string.
/// The resulting CSV string can be used as needed.
pub fn json_to_csv<'a>(json: Value) -> Result<String, TracebackError> {
    let mut wtr = csv::Writer::from_writer(vec![]);
    let zeroth = match json.get(0) {
        Some(zeroth) => zeroth,
        None => {
            return Err(traceback!("Failed to get zeroth element of json array")
                .with_extra_data(json!({ "json": json.to_string() })))
        }
    };
    let obj = match zeroth.as_object() {
        Some(obj) => obj,
        None => {
            return Err(
                traceback!("Failed to get zeroth element of json array as object")
                    .with_extra_data(json!({ "json": json.to_string() })),
            )
        }
    };
    let headers = obj.keys();
    let mut collected_headers: Vec<String> = headers
        .cloned()
        // sort alphabetically
        .collect::<Vec<String>>();
    collected_headers.sort();
    match wtr.write_record(&collected_headers) {
        Ok(_) => (),
        Err(e) => {
            return Err(traceback!("Failed to write CSV headers")
                .with_extra_data(json!({ "error": e.to_string() })))
        }
    }
    let arr = match json.as_array() {
        Some(arr) => arr,
        None => {
            return Err(traceback!("Failed to get json as array")
                .with_extra_data(json!({ "json": json.to_string() })))
        }
    };
    for record in arr {
        let mut row = Vec::new();
        for header in &collected_headers {
            let value = match record.get(header) {
                Some(value) => value,
                None => {
                    return Err(traceback!("Failed to get value from json record")
                        .with_extra_data(json!({ "json": json.to_string() })))
                }
            };
            match value.as_str() {
                Some(value) => row.push(value),
                None => {
                    return Err(
                        traceback!("Failed to parse value from json record as string")
                            .with_extra_data(json!({ "json": json.to_string() })),
                    )
                }
            };
        }
        match wtr.write_record(row) {
            Ok(_) => (),
            Err(e) => {
                return Err(traceback!("Failed to write CSV record")
                    .with_extra_data(json!({ "error": e.to_string() })))
            }
        };
    }
    let inner = match wtr.into_inner() {
        Ok(inner) => inner,
        Err(e) => {
            return Err(traceback!("Failed to convert CSV writer to inner")
                .with_extra_data(json!({ "error": e.to_string() })))
        }
    };
    match String::from_utf8(inner) {
        Ok(string) => Ok(string),
        Err(e) => {
            return Err(traceback!("Failed to convert CSV writer to string")
                .with_extra_data(json!({ "error": e.to_string() })))
        }
    }
}

/// This function takes in a csv file path and returns a serde_json::Value
/// NOTE: Some data will be lost in the conversion from csv to json.
/// This happens because serde_json automatically sorts the CSV headers alphabetically.
pub fn csv_file_to_json(path: &str) -> Result<serde_json::Value, TracebackError> {
    // read csv file, then pass it to csv_to_json
    let rdr = match csv::Reader::from_path(path) {
        Ok(rdr) => rdr,
        Err(e) => {
            return Err(traceback!("Failed to read CSV file")
                .with_extra_data(json!({ "error": e.to_string() })))
        }
    };
    match csv_to_json(rdr) {
        Ok(json) => Ok(json),
        Err(e) => Err(traceback!("Failed to parse CSV to json").with_parent(e)),
    }
}

pub struct Person {
    pub name: String,
    pub age: u8,
}

pub static BASIC_CSV: &str = r#"name,age
alice,20
bob,30
"#;

pub static BASIC_JSON: &str = r#"[{"name":"alice","age":"20"},{"name":"bob","age":"30"}]"#;

#[test]
fn test_csv_to_json() {
    let csv = Reader::from_reader(BASIC_CSV.as_bytes());
    let json = csv_to_json(csv);
    assert_eq!(
        json.unwrap(),
        serde_json::from_str::<Value>(BASIC_JSON).unwrap()
    );
}

#[test]
fn test_json_to_csv() {
    let json = serde_json::from_str::<Value>(BASIC_JSON).unwrap();
    let csv = json_to_csv(json);
    assert_eq!(csv.unwrap(), BASIC_CSV);
}
