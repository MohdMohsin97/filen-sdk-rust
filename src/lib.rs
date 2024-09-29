mod http;
mod crypto;
use crypto::derive_key_from_password;
use crate::http::ApiResponse;
use crate::http::Client;
use http::HttpMethod as method;
use serde::{Deserialize, Serialize};
use serde_json::from_reader;
use std::fs::File;
use std::io::BufReader;

#[derive(Debug, Serialize)]
struct InfoRequest {
    email: String,
}

#[derive(Debug, Serialize)]
struct LoginRequest {
    authVersion: i32,
    email: String,
    password: String,
    twoFactorCode: String,
    
}

#[derive(Deserialize, Debug)]
struct AuthInfo {
    email: String,
    password: String,
    salt: String,
    id: String,
    auth_key: String,
    api_key: String,
    master_key: String,
    public_key: String,
    private_key: String,
}

#[cfg(test)]

mod tests {
    use super::*;

    fn read_auth_info_from_file(file_path: &str) -> Result<AuthInfo, Box<dyn std::error::Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let auth_info: AuthInfo = from_reader(reader)?;
        Ok(auth_info)
    }
    #[test]
    fn test_auto_info_request() {
        let auth_info = read_auth_info_from_file("test_inputs.json").unwrap();
        let expected_response = ApiResponse {
            status: true,
            message: "Auth info fetched.".to_string(),
            code: "auth_info_fetched".to_string(),
            data: Some(serde_json::json!({
                "authVersion": 2,
                "email": auth_info.email,
                "id": 92022189,
                "salt": auth_info.salt,
            })),
        };
        let client = Client::new(String::from("CHIIIT"));
        let actual_response = client
            .request(
                method::POST,
                String::from("/v3/auth/info"),
                Some(InfoRequest {
                    email: auth_info.email.to_string(),
                }),
                None::<String>,
            )
            .unwrap();
        assert_eq!(actual_response.status, expected_response.status);
        assert_eq!(actual_response.message, expected_response.message);
        assert_eq!(actual_response.code, expected_response.code);
        assert_eq!(actual_response.data, expected_response.data);
    }

    #[test]
    fn test_login_request() {
        let auth_info = read_auth_info_from_file("test_inputs.json").unwrap();
        let expected_response = ApiResponse {
            status: true,
            message: "Login successful.".to_string(),
            code: "login_success".to_string(),
            data: Some(serde_json::json!({
                "apiKey": auth_info.api_key,
                "masterKeys": auth_info.master_key,
                "publicKey": auth_info.public_key,
                "privateKey": auth_info.private_key,
            })),
        };
        let client = http::Client::new(String::from("CHIIIT"));
        let actual_response = client.request(
            method::POST,
        String::from("/v3/login"),
        Some(LoginRequest {
            authVersion: 2,
            email: auth_info.email,
            password: auth_info.auth_key,
            twoFactorCode: String::from("XXXXXX"),
        }),
        None::<String>,
    ).unwrap();
        assert_eq!(actual_response.status, expected_response.status);
        assert_eq!(actual_response.message, expected_response.message);
        assert_eq!(actual_response.code, expected_response.code);
        assert_eq!(actual_response.data, expected_response.data);
    }

    #[test]
    fn test_base_folder() {

    }
}