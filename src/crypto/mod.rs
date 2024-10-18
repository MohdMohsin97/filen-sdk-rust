mod utils;

use aes_gcm::aead::{rand_core::RngCore, OsRng};
use base64::prelude::*;
use hex::encode;
use ring::pbkdf2::{Algorithm, PBKDF2_HMAC_SHA512};
use std::error::Error;
use utils::{run_aes_gcm_decryption, run_aes_gcm_encryprion, run_pbkdf2, run_sha512};
struct DrivedMasterKeyAndPassword {
    master_key: String,
    derived_password: String,
}

pub fn derive_key_from_password(
    password: String,
    salt: String,
    hash: Algorithm,
    iterations: u32,
    bit_length: usize,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let key = run_pbkdf2(password, salt, hash, iterations, bit_length)?;

    Ok(key)
}

fn generate_password_and_master_key(
    raw_password: String,
    salt: String,
) -> Result<DrivedMasterKeyAndPassword, Box<dyn Error>> {
    let hash = derive_key_from_password(raw_password, salt, PBKDF2_HMAC_SHA512, 200_000, 512)?;

    let drive_key = hex::encode(hash);

    let master_key = &drive_key[0..(drive_key.len() / 2)];
    let derived_password = &drive_key[(drive_key.len() / 2)..];

    let master_key = master_key.to_string();
    let derived_password = run_sha512(derived_password.to_owned());

    Ok(DrivedMasterKeyAndPassword {
        master_key,
        derived_password,
    })
}

fn encrypt_metadata(metadata: String, key: [u8; 32]) -> Result<String, Box<dyn Error>> {
    let mut nonce = [0u8; 12];
    OsRng.fill_bytes(&mut nonce);

    let result = run_aes_gcm_encryprion(&metadata.as_bytes(), &key, &nonce)?;
    
    let result_str = BASE64_STANDARD.encode(&result);
    
    // // SAFETY: 
    let nonce_str = unsafe {
        String::from_utf8_unchecked(nonce.to_vec())
    };

    Ok(format!("{}{}{}","002", nonce_str, result_str))
}

fn decrypt_metadata(metadata: String, key: [u8; 32]) -> Result<String, Box<dyn Error>> {
    let nonce_str = &metadata[3..15];

    let nonce: &[u8; 12] = nonce_str.as_bytes().try_into()?;

    let encrypted = BASE64_STANDARD.decode(&metadata[15..])?;

    let result = run_aes_gcm_decryption(&encrypted, &key, nonce)?;

    let result_str = String::from_utf8(result)?;

    Ok(result_str)
}

#[cfg(test)]
mod test {
    use super::*;
    use aes_gcm::aead::{rand_core::RngCore, OsRng};
    use serde::Deserialize;
    use serde_json::from_reader;
    use std::{fs::File, io::BufReader};
    use utils::{generate_random_key, run_aes_gcm_decryption, run_aes_gcm_encryprion};

    #[derive(Deserialize, Debug)]
    struct TestInfo {
        email: String,
        password: String,
        salt: String,
        auth_key: String,
    }

    fn read_test_info_from_file(file_path: &str) -> Result<TestInfo, Box<dyn std::error::Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let auth_info: TestInfo = from_reader(reader)?;
        Ok(auth_info)
    }

    #[test]
    fn test_auth_key_from_raw_password() {
        let test_info = read_test_info_from_file("test_inputs.json").unwrap();

        let keys = generate_password_and_master_key(test_info.password, test_info.salt).unwrap();

        assert_eq!(test_info.auth_key, keys.derived_password);
    }

    #[test]
    fn test_encrypt_decrypt_chunk() {
        let data = b"Hello, this is a test message!";
        let mut nonce = [0u8; 12];
        OsRng.fill_bytes(&mut nonce);

        let key = generate_random_key();

        let encrypt_data = run_aes_gcm_encryprion(data, &key, &nonce).unwrap();

        let decrypt_data = run_aes_gcm_decryption(&encrypt_data, &key, &nonce).unwrap();

        assert_eq!(decrypt_data, data);
    }

    #[test]
    fn test_metadata_encrypt_decrypt() {
        let metadata = String::from("MetaData");

        let key = generate_random_key();

        let encypt_metadata = encrypt_metadata(metadata.to_owned(), key).unwrap();

        let decrypt_metadata = decrypt_metadata(encypt_metadata, key).unwrap();

        assert_eq!(metadata, decrypt_metadata);
    }
}
