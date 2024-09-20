use serde::{Deserialize, Serialize};

struct BaseUrlChooser;

impl BaseUrlChooser {
    fn new() -> Self {
        BaseUrlChooser
    }

    fn api_url(&self) -> url::Url {
        // Randomly chose one of 6 API endpoints
        url::Url::parse("https://api.filen.io").unwrap()
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct InfoResponse {
    email: String,
    auth_version: i32,
    salt: String,
    id: i64,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum DataVariant {
    Info(InfoResponse),
}

#[derive(Deserialize)]
struct GenericRespone {
    status: bool,
    message: String,
    code: String,
    data: DataVariant,
}

#[derive(Serialize)]
struct InfoRequest {
    email: String,
}

struct ApiClient {
    api_key: Option<String>,
    base_url: BaseUrlChooser,
}

impl ApiClient {
    pub fn new_with_api_key(api_key: &str) -> Self {
        Self {
            api_key: Some(api_key.to_string()),
            base_url: BaseUrlChooser,
        }
    }

    pub fn new() -> Self {
        Self {
            api_key: None,
            base_url: BaseUrlChooser,
        }
    }

    pub fn get_info(&self, email: &str) -> Result<InfoResponse, String> {
        // Call /info endpoint and return Salt
        let info_req = InfoRequest {
            email: email.to_string(),
        };

        let json_value = serde_json::to_value(&info_req).unwrap();
        println!("JSON: {:?}", json_value);
        Ok(InfoResponse {
            email: "dummy@email.com".to_string(),
            auth_version: 0,
            salt: "dummy".to_string(),
            id: 0,
        })
    }
}

// #[cfg(test)]
mod test {

    #[test]
    fn info_api() {
        let api_client = super::ApiClient::new();

        let resposne = api_client.get_info("dummy@email.com");

        // let test_config = read_rom.then_parse_into_toml_struct;

        let expected_response = super::InfoResponse {
            email: "dummy@email.com".to_string(),
            auth_version: 0,
            salt: "dummy".to_string(),
            id: 0,
        };

        assert!(resposne.is_ok());

        assert_eq!(resposne.unwrap(), expected_response);
    }
}
