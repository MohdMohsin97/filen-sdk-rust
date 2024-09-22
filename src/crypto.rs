use ring::pbkdf2;
use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;
use base64::{engine::general_purpose::STANDARD};

#[derive(Debug, Serialize, Deserialize)]
pub struct DeriveKeyFromPasswordParams {
    pub password: String,
    pub salt: String,
    pub iterations: u32,
    pub hash: String,
    pub bit_length: usize,
    pub return_hex: bool,
    pub environment: String,
}

pub fn derive_key_from_password(params: DeriveKeyFromPasswordParams) -> Result<String, Err> {
    // if params.environment == "node" {
        let salt_bytes = params.salt.as_bytes();
        let password_bytes = params.password.as_bytes();
        let mut key = vec![0u8; params.bit_length / 8];
        let iterations = NonZeroU32::new(params.iterations).ok_or("Invalid iterations")?;

        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA512,
            iterations,
            salt_bytes,
            password_bytes,
            &mut key,
        );

        let key = hex::encode(key);
        Ok(key)
        // } else {
        //     return Ok(base64::Engine::encode(key));
        // }
    // } else if params.environment == "browser" {
    //     let client = Client::new();
    //     let response = client.post("https://example.com/derive-key")
    //         .json(&params)
    //         .send()?
    //         .json::<Value>()?;

    //     if params.return_hex {
    //         return Ok(response["key"].as_str().unwrap().to_string());
    //     } else {
    //         return Ok(base64::encode(&self, key));
    //     }
    // }
}

