use csv::Reader;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};

use crate::error_types::TracebackError;

/// This function takes in a csv::Reader<&[u8]> and returns a serde_json::Value
/// It assumes that the first row of the csv is the header row, and that all
/// other rows are data rows.
/// It also assumes that all data is in the form of strings.
/// Should these assumptions not be true, this function will return an error.
/// NOTE: Some data will be lost in the conversion from csv to json.
/// This happens because serde_json automatically sorts the CSV headers alphabetically.
/// #PANICS
/// This function should not panic under any circumstances, verified by no_panic.
#[no_panic::no_panic]
pub fn csv_to_json<T: std::io::Read>(
    mut csv: Reader<T>,
) -> Result<serde_json::Value, TracebackError> {
    let headers = match csv.headers().cloned() {
        Ok(headers) => headers,
        Err(e) => {
            return Err(TracebackError::new(
                "Failed to read CSV headers".to_string(),
                file!().to_string(),
                line!(),
            )
            .with_extra_data(json!({ "error": e.to_string() })))
        }
    };
    let mut records = Vec::new();
    for result in csv.records() {
        let record = match result {
            Ok(record) => record,
            Err(e) => {
                return Err(TracebackError::new(
                    "Failed to read CSV record".to_string(),
                    file!().to_string(),
                    line!(),
                )
                .with_extra_data(json!({ "error": e.to_string() })))
            }
        };
        let mut obj = serde_json::Map::new();
        for (i, header) in headers.iter().enumerate() {
            obj.insert(
                header.to_string(),
                serde_json::Value::String(record[i].to_string()),
            );
        }
        records.push(serde_json::Value::Object(obj));
    }
    Ok(serde_json::Value::Array(records))
}

pub fn json_to_csv<'a>(json: Value) -> Result<String, TracebackError> {
    let mut wtr = csv::Writer::from_writer(vec![]);
    let headers = json[0].as_object().unwrap().keys();
    let mut collected_headers: Vec<String> = headers
        .cloned()
        // sort alphabetically
        .collect::<Vec<String>>();
    collected_headers.sort();
    wtr.write_record(&collected_headers).unwrap();
    for record in json.as_array().unwrap() {
        let mut row = Vec::new();
        for header in &collected_headers {
            row.push(record[header].as_str().unwrap());
        }
        wtr.write_record(row).unwrap();
    }
    let inner = match wtr.into_inner() {
        Ok(inner) => inner,
        Err(e) => {
            return Err(TracebackError::new(
                "Failed to convert CSV writer to inner".to_string(),
                file!().to_string(),
                line!(),
            )
            .with_extra_data(json!({ "error": e.to_string() })))
        }
    };
    match String::from_utf8(inner) {
        Ok(string) => Ok(string),
        Err(e) => {
            return Err(TracebackError::new(
                "Failed to convert CSV writer to string".to_string(),
                file!().to_string(),
                line!(),
            )
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
            return Err(TracebackError::new(
                "Failed to read CSV file".to_string(),
                file!().to_string(),
                line!(),
            )
            .with_extra_data(json!({ "error": e.to_string() })))
        }
    };
    match csv_to_json(rdr) {
        Ok(json) => Ok(json),
        Err(e) => Err(TracebackError::new(
            "Failed to parse CSV to json".to_string(),
            file!().to_string(),
            line!(),
        )
        .with_parent(e)),
    }
}

#[no_panic::no_panic]
pub fn panics() {
    panic!()
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
