use rand::seq::SliceRandom;
use rand::thread_rng;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::{Client, Response};
use serde_json::Value;
use std::error::Error;
use std::time::Duration;

struct APIClientConfig {
    api_key: String,
}

enum HttpMethod {
    Get,
    Post,
}

#[derive(serde::Serialize)]
struct BaseRequestParameters {
    endpoint: String,
    url: Option<String>,
    abort_signal: Option<tokio::sync::oneshot::Receiver<()>>,
    timeout: Option<Duration>,
    max_retries: Option<u32>,
    retry_timeout: Option<Duration>,
    response_type: Option<ResponseType>,
    headers: Option<HeaderMap>,
    // Functions cannot be serialised. find a workaround
    on_upload_progress: Option<Box<dyn Fn(u64, u64) + Send>>,
    on_download_progress: Option<Box<dyn Fn(u64, u64) + Send>>,
    api_key: Option<String>,
}

enum ResponseType {
    Json,
    Text,
    Binary,
}

struct GetRequestParameters {
    base: BaseRequestParameters,
    method: HttpMethod,
    include_raw: Option<bool>,
}

struct PostRequestParameters {
    base: BaseRequestParameters,
    method: HttpMethod,
    // XXX:
    // data can be one of many types, represent those in an enum and flatten it. Using dyn Any
    // comes with no benefit, use strict typing and represent all request _types_ possible.
    // Consider this pseudocode
    // data: PostRquestData {
    //     UplaodRequest(entitiy)
    //     DownlaodRequest(entitiy)
    //     ...
    // }
    data: Box<dyn std::any::Any + Send>,
    include_raw: Option<bool>,
}

enum RequestParameters {
    Get(GetRequestParameters),
    Post(PostRequestParameters),
}

impl GetRequestParameters {
    fn new(base: BaseRequestParameters, include_raw: Option<bool>) -> Self {
        GetRequestParameters {
            base,
            method: HttpMethod::Get,
            include_raw,
        }
    }
}

impl PostRequestParameters {
    fn new(
        base: BaseRequestParameters,
        data: Box<dyn std::any::Any + Send>,
        include_raw: Option<bool>,
    ) -> Self {
        PostRequestParameters {
            base,
            method: HttpMethod::Post,
            data,
            include_raw,
        }
    }
}

struct UploadChunkResponse {
    bucket: String,
    region: String,
}

struct APIClientDefaults {
    gateway_urls: &'static [&'static str],
    egest_urls: &'static [&'static str],
    ingest_urls: &'static [&'static str],
    gateway_timeout: Duration,
    egest_timeout: Duration,
    ingest_timeout: Duration,
    max_retries: u32,
    retry_timeout: Duration,
}

static API_CLIENT_DEFAULTS: APIClientDefaults = APIClientDefaults {
    gateway_urls: &[
        "https://gateway.filen.io",
        "https://gateway.filen.net",
        "https://gateway.filen-1.net",
        "https://gateway.filen-2.net",
        "https://gateway.filen-3.net",
        "https://gateway.filen-4.net",
        "https://gateway.filen-5.net",
        "https://gateway.filen-6.net",
    ],
    egest_urls: &[
        "https://egest.filen.io",
        "https://egest.filen.net",
        "https://egest.filen-1.net",
        "https://egest.filen-2.net",
        "https://egest.filen-3.net",
        "https://egest.filen-4.net",
        "https://egest.filen-5.net",
        "https://egest.filen-6.net",
    ],
    ingest_urls: &[
        "https://ingest.filen.io",
        "https://ingest.filen.net",
        "https://ingest.filen-1.net",
        "https://ingest.filen-2.net",
        "https://ingest.filen-3.net",
        "https://ingest.filen-4.net",
        "https://ingest.filen-5.net",
        "https://ingest.filen-6.net",
    ],

    gateway_timeout: Duration::from_millis(300000),
    egest_timeout: Duration::from_millis(1800000),
    ingest_timeout: Duration::from_millis(3600000),
    max_retries: 64,
    retry_timeout: Duration::from_millis(1000),
};

struct APIClient {
    client: Client,
    config: APIClientConfig,
}

impl APIClient {
    pub fn new(config: APIClientConfig) -> Self {
        APIClient {
            client: Client::new(),
            config,
        }
    }

    fn build_headers(&self, params: Option<String>) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            reqwest::header::AUTHORIZATION,
            format!(
                "Bearer {}",
                params.unwrap_or_else(|| self.config.api_key.clone())
            )
            .parse()
            .unwrap(),
        );
        headers.insert(
            reqwest::header::ACCEPT,
            "application/json, text/plain, */*".parse().unwrap(),
        );
        headers
    }

    //Implement this :
    //...(environment === "node" ? { "User-Agent": "filen-sdk" } : {})

    /*async fn post(&self, params: PostRequestParameters) {
        let mut _headers = if let Some(h) = &params.base.headers {
            h.clone()
        } else {
            self.build_headers(Some(params.base.api_key.unwrap().clone()))
        };
    }*/
    async fn post(&self, params: PostRequestParameters) -> Result<Response, Box<dyn Error>> {
        let client = Client::new();
        let mut headers = if let Some(h) = params.base.headers {
            h
        } else {
            self.build_headers(params.base.api_key.clone())
        };

        if let Some(api_key) = &params.base.api_key {
            if !headers.contains_key(AUTHORIZATION) {
                headers.insert(
                    AUTHORIZATION,
                    HeaderValue::from_str(&format!("Bearer {}", api_key)).unwrap(),
                );
            }
        }
        let mut rng = thread_rng();

        let url = params.base.url.unwrap_or_else(|| {
            API_CLIENT_DEFAULTS
                .gateway_urls
                .choose(&mut rng)
                .unwrap()
                .to_string()
        });
        //use match here, remove unwrap

        if url.is_empty() {
            return Err("No URL.".into());
        }

        // XXX: This is (kinda)impossible in rust and a bad design pattern (Reason to avoid any).
        // This can be made into
        //
        // match params.data {
        //   UplaodRequest(req) => req.do();
        //   ..
        // }
        let post_data_is_buffer = params.data.is_array() || params.data.is_object();

        if params.base.headers.is_none() && !post_data_is_buffer {
            let checksum = buffer_to_hash(&params.data).await?;
            headers.insert("Checksum", HeaderValue::from_str(&checksum).unwrap());
        }

        let timeout = params.base.timeout.unwrap_or(Duration::from_millis(3000));
        let request = client
            .post(&format!("{}{}", url, params.base.endpoint))
            .headers(headers)
            .timeout(timeout);

        let request = if post_data_is_buffer {
            match params.data.downcast_ref::<String>() {
                Some(s) => request.body(s.clone()),
                None => return Err("Expected data to be a String".into()),
            }
        } else {
            request.json(&params.data)
        };

        let response = request.send().await?;

        if response.status().is_success() {
            if let Some(response_type) = params.base.response_type {
                // XXX: Incompatible types cannot be compared
                if response_type == "stream" {
                    // Handle stream response
                    unimplemented!("Stream ahndling not implemented");
                } else if response_type == "json" {
                    let json: Value = response.json().await?;
                    println!("Response JSON: {:?}", json);
                }
            }
        } else {
            println!("Request failed with status: {}", response.status());
        }

        Ok(())
    }
}

async fn buffer_to_hash(data: &Value) -> Result<String, Box<dyn Error>> {
    use sha2::{Digest, Sha512};
    let mut hasher = Sha512::new();
    hasher.update(data.to_string().as_bytes());
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

//Copilot generated main function
// #[tokio::main]
// async fn main() {
//     let client = APIClient {
//         config: Config {
//             api_key: "your_api_key".to_string(),
//         },
//     };

//     let params = PostRequestParameters {
//         base: BaseParams {
//             api_key: Some("your_api_key".to_string()),
//             url: None,
//         },
//         data: serde_json::json!({ "key": "value" }),
//         headers: None,
//         timeout: Some(30000),
//         response_type: Some("json".to_string()),
//         endpoint: "/your-endpoint".to_string(),
//     };

//     match client.post(params).await {
//         Ok(_) => println!("Request successful"),
//         Err(e) => println!("Request failed: {}", e),
//     }
// }
