use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
};

use serde_json::json;

use crate::{error_types::TracebackError, traceback};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
    HEAD,
    PATCH,
}

impl Display for Method {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Method::GET => write!(f, "GET"),
            Method::POST => write!(f, "POST"),
            Method::PUT => write!(f, "PUT"),
            Method::DELETE => write!(f, "DELETE"),
            Method::HEAD => write!(f, "HEAD"),
            Method::PATCH => write!(f, "PATCH"),
        }
    }
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
            return Err(
                traceback!(e, "Error building request").with_extra_data(json!({
                    "url": url,
                    "headers": headers,
                    "body": body
                })),
            );
        }
    };
    let response = match client.execute(request).await {
        Ok(r) => r,
        Err(e) => {
            return Err(
                traceback!(e, "Error executing request").with_extra_data(json!({
                    "url": url,
                    "headers": headers,
                    "body": body
                })),
            );
        }
    };
    let response = match response.text().await {
        Ok(r) => r,
        Err(e) => {
            return Err(
                traceback!(e, "Error reading response").with_extra_data(json!({
                    "url": url,
                    "headers": headers,
                    "body": body
                })),
            );
        }
    };
    let response: T = match serde_json::from_str(&response) {
        Ok(r) => r,
        Err(e) => {
            return Err(
                traceback!(e, "Error parsing response").with_extra_data(json!({
                    "url": url,
                    "headers": headers,
                    "body": body,
                    "response": response
                })),
            );
        }
    };

    Ok(response)
}
