use std::string;
use std::io::{self, Write};
use crate::http;
use crate::crypto;
use rpassword::read_password;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct InfoRequest {
    email: String,
}

#[derive(Debug, Deserialize)]
struct AuthInfoResponseData {
    authVersion: i32,
    email: String,
    id: i32,
    salt: String,
}

#[derive(Debug, Serialize)]
struct LoginRequest {
    authVersion: i32,
    email: String,
    password: String,
    twoFactorCode: String,
}

#[derive(Debug, Deserialize)]
struct LoginResponseData {
    #[serde(rename = "apiKey")]
    api_key: String,
    #[serde(rename = "masterKeys")]
    master_key: String,
    #[serde(rename = "publicKey")]
    public_key: String,
    #[serde(rename = "privateKey")]
    private_key: String,
}

#[derive(Debug, Deserialize)]
struct UserBaseFolderResponseData {
    userBaseFolder: String,
}

pub fn get_email() {
    let mut email  = String::new();
    println!("Enter your email: ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut email).expect("Failed to read email");
    let email = email.trim().to_string();

    println!("Enter your password: ");
    let password = read_password().expect("Failed to read password");

    let client = http::Client::new(String::from("CHIIIT"));
    let info_request = InfoRequest {
        email: email
    };

    let auth_info_response = client.request(http::HttpMethod::POST, String::from("/v3/auth/info"), Some(info_request),None::<String>).unwrap();
    let auth_info_parsed : AuthInfoResponseData  = serde_json::from_value(auth_info_response.data.unwrap()).unwrap();
    
    let login_request = LoginRequest {
        authVersion: auth_info_parsed.authVersion,
        email: auth_info_parsed.email,
        password: crypto::derive_key_from_password(password, auth_info_parsed.salt, ring::pbkdf2::PBKDF2_HMAC_SHA512, 200000, 512).unwrap(),
        twoFactorCode: String::from("XXXXXX"),
    };

    let login_response = client.request(http::HttpMethod::POST, String::from("/v3/login"), Some(login_request), None::<String>).unwrap();
    let login_response_parsed : LoginResponseData = serde_json::from_value(login_response.data.unwrap()).unwrap();

    let user_base_folder_response = client.request(http::HttpMethod::GET, String::from("/v3/user/baseFolder"), None::<()>,Some(login_response_parsed.api_key)).unwrap();

    }

