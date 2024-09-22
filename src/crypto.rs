use ring::pbkdf2;
use ring::digest::{Context, SHA512};
use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;
use std::error::Error;

static PBKDF2_ALG: pbkdf2::Algorithm = pbkdf2::PBKDF2_HMAC_SHA512;

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

pub fn derive_key_from_password(params: DeriveKeyFromPasswordParams) -> Result<String, Box<dyn Error>> {
    // if params.environment == "node" {
        let salt_bytes = params.salt.as_bytes();
        let password_bytes = params.password.as_bytes();
        let mut key = vec![0u8; params.bit_length / 8];
        let iterations = NonZeroU32::new(params.iterations).ok_or("Invalid iterations")?;

        pbkdf2::derive(
            PBKDF2_ALG,
            iterations,
            salt_bytes,
            password_bytes,
            &mut key,
        );
        let first_half = &key[0..(params.bit_length / 16)];
        let second_half = &key[(params.bit_length / 16)..];
        
        println!("{:?}", hex::encode(second_half));
        let second_half = hex::encode(second_half);

        let mut context = Context::new(&SHA512);
        context.update(second_half.as_bytes());
        let digest = context.finish();
        let final_key = hex::encode(digest.as_ref());
        if !final_key.is_empty() {
            return Ok(final_key);
        }
        Err(format!("crypto.utils.deriveKeyFromPassword not implemented for {} environment", params.environment).into())
}


