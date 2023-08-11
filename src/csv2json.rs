use csv::Reader;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;

pub fn csv_to_json(mut csv: Reader<&[u8]>) -> serde_json::Value {
    let headers = csv.headers().cloned().unwrap();
    let mut records = Vec::new();
    for result in csv.records() {
        let record = result.unwrap();
        let mut obj = serde_json::Map::new();
        for (i, header) in headers.iter().enumerate() {
            obj.insert(
                header.to_string(),
                serde_json::Value::String(record[i].to_string()),
            );
        }
        records.push(serde_json::Value::Object(obj));
    }
    serde_json::Value::Array(records)
}

pub fn json_to_csv<'a>(json: Value) -> String {
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
    String::from_utf8(wtr.into_inner().unwrap()).unwrap()
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
    assert_eq!(json, serde_json::from_str::<Value>(BASIC_JSON).unwrap());
}

#[test]
fn test_json_to_csv() {
    let json = serde_json::from_str::<Value>(BASIC_JSON).unwrap();
    let csv = json_to_csv(json);
    assert_eq!(csv, BASIC_CSV);
}
