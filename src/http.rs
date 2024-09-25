use reqwest::blocking::Client as HttpClient;
use serde::{Deserialize, Serialize};
use serde_json::{to_vec, Value};
use std::{error::Error, fmt::Debug, time::Duration};

const GATEWAY_URL: &str = "https://gateway.filen.io";

#[derive(Debug, Serialize, Clone, Copy)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
}

impl HttpMethod {
    fn as_str(&self) -> &str {
        match self {
            HttpMethod::GET => "GET",
            HttpMethod::POST => "POST",
            HttpMethod::PUT => "PUT",
            HttpMethod::DELETE => "DELETE",
        }
    }

    fn to_reqwest_method(&self) -> reqwest::Method {
        match self {
            HttpMethod::GET => reqwest::Method::GET,
            HttpMethod::POST => reqwest::Method::POST,
            HttpMethod::PUT => reqwest::Method::PUT,
            HttpMethod::DELETE => reqwest::Method::DELETE,
        }
    }
}

#[derive(Debug)]
pub struct RequestError {
    message: String,
    method: HttpMethod,
    path: String,
    source: Box<dyn Error>,
}

#[derive(Debug)]
struct LoginResponse {
    api_key: String,
    master_key: String,
    public_key: String,
    private_key: String,
}

impl RequestError {
    fn new(message: String, method: HttpMethod, path: String, source: Box<dyn Error>) -> RequestError {
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

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiResponse {
    pub status: bool,
    pub message: String,
    pub code: String,
    pub data: Option<Value>,
}

pub struct Client {
    api_key: Option<String>,
}

impl Client {
    pub fn new(key: String) -> Client {
        Client { api_key: Some(key) }
    }

    pub fn request<T: Serialize + Debug>(
        &self,
        method: HttpMethod,
        path: String,
        request: Option<T>,
    ) -> Result<ApiResponse, RequestError> {
        // Marshalled request body
        let marshalled = if let Some(req_body) = request {
            match to_vec(&req_body) {
                Ok(json) => json,
                Err(err) => {
                    return Err(RequestError::new(
                        format!("Cannot marshal request body: {:?}", req_body),
                        method,
                        path.to_string(),
                        Box::new(err),
                    ));
                }
            }
        
        } else {
            vec![]
        };
        // Build request
        let url = format!("{}{}", GATEWAY_URL, path);
        let client = HttpClient::new();
        let req = client
            .request(method.to_reqwest_method(), url)
            .body(marshalled)
            .header("Content-Type", "application/json");

        // // Set Authorization header if API key is present
        // if let Some(ref api_key) = self.api_key {
        //     req = req.header("Authorization", format!("Bearer {}", api_key));
        // }
        // Send the request
        let response = req.timeout(Duration::from_secs(10)).send().map_err(|err| {
            RequestError::new(
                "Cannot send request".to_string(),
                method,
                path.to_string(),
                Box::new(err),
            )
        })?;

        // Read response body
        let res_body = response.bytes().map_err(|err| {
            RequestError::new(
                "Cannot read response body".to_string(),
                method,
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
                method,
                path.to_string(),
                Box::new(err),
            )
        })?;

        Ok(api_response)
    }
}