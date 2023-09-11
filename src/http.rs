use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
};

use serde_json::json;

use traceback_error::{traceback, TracebackError};

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

/// Attempts to fetch data from a given URL using an HTTP request, and then parses the response into a specified type.
///
/// # Arguments
///
/// * `url` - A string representing the URL to fetch data from.
/// * `headers` - An optional `HashMap` containing HTTP headers as key-value pairs.
/// * `body` - An optional string containing the request body data.
/// * `method` - An HTTP request method from the `Method` enum (e.g., `Method::GET`, `Method::POST`).
///
/// # Returns
///
/// * `Result<T, TracebackError>` - A `Result` containing the parsed response data of type `T` if the operation is successful,
///   or an error message as a `TracebackError` if there's an issue during the process.
///
/// # Type Parameters
///
/// * `T` - The type into which the response data should be deserialized. It must implement `serde::de::DeserializeOwned`.
///
/// # Example
///
/// ```rust
/// use std::collections::HashMap;
/// use serde::Deserialize;
/// use traceback_error::TracebackError;
/// use your_module_name::{attempt_fetch_and_parse, Method};
///
/// #[derive(Debug, Deserialize)]
/// struct Post {
///     userId: u32,
///     id: u32,
///     title: String,
///     body: String,
/// }
///
/// #[tokio::main]
/// async fn main() -> Result<(), TracebackError> {
///     let url = "https://jsonplaceholder.typicode.com/posts/1";
///     let mut headers = HashMap::new();
///     headers.insert("Content-Type", "application/json");
///
///     let post: Post = attempt_fetch_and_parse(url, &Some(headers), None, Method::GET).await?;
///
///     println!("{:?}", post);
///
///     Ok(())
/// }
/// ```
///
/// In this example, the function `attempt_fetch_and_parse` is used to fetch JSON data from a URL, and the response is deserialized into a `Post` struct.
/// The result is then printed.
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
