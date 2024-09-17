use reqwest::blocking::Client as HttpClient;
use serde::{Deserialize, Serialize};
use serde_json::{to_vec, Value};
use std::{error::Error, fmt::Debug, time::Duration};

#[allow(non_camel_case_types)]
const GATEWAY_URL: &str = "https://gateway.filen.io";

#[derive(Serialize, Deserialize, Debug)]
struct ApiResponse {
    status: bool,
    message: String,
    code: String,
    data: Option<Value>,
}

#[derive(Debug)]
struct RequestError {
    message: String,
    method: String,
    path: String,
    source: Box<dyn Error>,
}

impl RequestError {
    fn new(message: String, method: String, path: String, source: Box<dyn Error>) -> RequestError {
        RequestError {
            message,
            method,
            path,
            source,
        }
    }
}

impl std::fmt::Display for RequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RequestError: {}", self.message)
    }
}

impl Error for RequestError {}

struct Client {
    api_key: String,
}

impl Client {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }

    pub fn request<T: Serialize + Debug>(
        &self,
        method: String,
        path: String,
        request: Option<T>,
    ) -> Result<ApiResponse, RequestError> {
        // Marshalled request body
        let marshalled = if let Some(req_body) = request {
            match to_vec(&req_body) {
                Ok(json) => json,
                Err(err) => {
                    return Err(RequestError::new(
                        format!("Cannot marshel request body: {:?}", req_body),
                        method.to_string(),
                        path.to_string(),
                        Box::new(err),
                    ));
                }
            }
        } else {
            vec![]
        };

        // build request
        let url = format!("{}{}", GATEWAY_URL, path);
        let client = HttpClient::new();
        let mut req = client
            .request(method.parse().unwrap_or_else(|_| reqwest::Method::GET), url)
            .body(marshalled)
            .header("Content-Type", "application/json");

        // Set Authorization header if API key is present
        if !self.api_key.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", self.api_key))
        }

        // Send the request
        let response = req.timeout(Duration::from_secs(10)).send().map_err(|err| {
            RequestError::new(
                "Cannot send request".to_string(),
                method.to_string(),
                path.to_string(),
                Box::new(err),
            )
        })?;

        // Read response body
        let res_body = Vec::new();
        response.bytes().map_err(|err| {
            RequestError::new(
                "Cannot read response body".to_string(),
                method.to_string(),
                path.to_string(),
                Box::new(err),
            )
        })?;

        // Parse the response
        let api_response: ApiResponse = serde_json::from_slice(&res_body).map_err(|err| {
            RequestError::new(
                format!(
                    "Cannot unmarshal response {}",
                    String::from_utf8_lossy(&res_body)
                ),
                method.to_string(),
                path.to_string(),
                Box::new(err),
            )
        })?;

        Ok(api_response)
    }
}

