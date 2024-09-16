use reqwest::{Client, Response};
use std::collections::HashMap;
use std::time::Duration;
use reqwest::header::HeaderMap;


struct APIClientConfig {
    apiKey: String,
}

enum HttpMethod {
    Get,
    Post,
}

struct BaseRequestParameters {
    endpoint: String,
    url: Option<String>,
    abort_signal: Option<tokio::sync::oneshot::Receiver<()>>,
    timeout: Option<Duration>,
    max_retries: Option<u32>,
    retry_timeout: Option<Duration>,
    response_type: Option<ResponseType>,
    headers: Option<HeaderMap>,
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
    data: Box<dyn std::any::Any + Send>,
    include_raw: Option<bool>,
}

enum RequestParameters {
    Get(GetRequestParameters),
    Post(PostRequestParameters),
}

impl GetRequestParameters {
    fn new(base: BaseRequestParameters, include_raw: Option<bool>) -> Self {
        GetRequestParameters { base, method: HttpMethod::Get, include_raw }
    }
}

impl PostRequestParameters {
    fn new(base: BaseRequestParameters, data: Box<dyn std::any::Any + Send>, include_raw: Option<bool>) -> Self {
        PostRequestParameters { base, method: HttpMethod::Post,data,  include_raw }
    }
}

struct UploadChunkResponse {
    bucket: String,
    region: String,
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

    fn build_headers(&self, params: Option<&HashMap<String, String>>) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert(
            "Authorization".to_string(),
            format!(
                "Bearer {}",
                params
                    .and_then(|p| p.get("api_key"))
                    .unwrap_or(&self.config.apiKey)
            ),
        );
        headers.insert("Accept".to_string(), "application/json, text/plain, */*".to_string());
        headers
    }

    async fn post(&self, params: PostRequestParameters) {
        let mut headers = if let Some(h) = &params.base.headers {
            h.clone()
        } else {
            self.build_headers(Some(&params.base.api_key))
        };
    }
}