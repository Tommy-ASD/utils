use std::collections::HashMap;

use serde_json::json;

use crate::{error_types::TracebackError, traceback};

pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
    HEAD,
    PATCH,
}

pub async fn attempt_fetch_and_parse<T>(
    url: &str,
    headers: &Option<HashMap<&str, &str>>,
    body: Option<&str>,
    method: Method,
) -> Result<T, TracebackError>
where
    T: serde::de::DeserializeOwned,
{
    let client = reqwest::Client::new();
    let mut request_builder = match method {
        Method::GET => client.get(url),
        Method::POST => client.post(url),
        Method::PUT => client.put(url),
        Method::DELETE => client.delete(url),
        Method::HEAD => client.head(url),
        Method::PATCH => client.patch(url),
    };
    if let Some(h) = headers {
        for (k, v) in h {
            request_builder = request_builder.header(k.to_string(), v.to_string());
        }
    }
    let request = match &body {
        Some(b) => request_builder.body(b.to_string()).build(),
        None => request_builder.build(),
    };
    let request = match request {
        Ok(r) => r,
        Err(e) => {
            return Err(traceback!("Error building request").with_extra_data(json!({
                "url": url,
                "error": format!("{}", e),
                "headers": headers,
                "body": body
            })));
        }
    };
    let response = match client.execute(request).await {
        Ok(r) => r,
        Err(e) => {
            return Err(
                traceback!("Error executing request").with_extra_data(json!({
                    "url": url,
                    "error": format!("{}", e),
                    "headers": headers,
                    "body": body
                })),
            );
        }
    };
    let response = match response.text().await {
        Ok(r) => r,
        Err(e) => {
            return Err(traceback!("Error reading response").with_extra_data(json!({
                "url": url,
                "error": format!("{}", e),
                "headers": headers,
                "body": body
            })));
        }
    };
    let response: T = match serde_json::from_str(&response) {
        Ok(r) => r,
        Err(e) => {
            return Err(traceback!("Error parsing response").with_extra_data(json!({
                "url": url,
                "error": format!("{}", e),
                "headers": headers,
                "body": body,
                "response": response
            })));
        }
    };

    Ok(response)
}
