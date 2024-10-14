use aes_gcm::aead::rand_core::RngCore;
use aes_gcm::aead::{Aead, OsRng};
use aes_gcm::{Aes256Gcm, Key, KeyInit, Nonce};
use ring::digest::{self, digest};
use ring::pbkdf2::{self, Algorithm, PBKDF2_HMAC_SHA512};
use std::error::Error;
use std::num::NonZeroU32;

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

fn run_pbkdf2(
    password: String,
    salt: String,
    hash: Algorithm,
    iterations: u32,
    bit_length: usize,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut key = vec![0u8; bit_length / 8];

    pbkdf2::derive(
        hash,
        NonZeroU32::new(iterations).expect("Invalid Iterations"),
        salt.as_bytes(),
        password.as_bytes(),
        &mut key,
    );

    Ok(key)
}

fn run_sha512(data: String) -> String {
    let hex = digest(&digest::SHA512, data.as_bytes());

    hex::encode(hex)
}

fn generate_random_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}

fn run_aes_gcm_encryprion(
    data: &[u8],
    key: &[u8; 32],
    nonce: &[u8; 12],
) -> Result<Vec<u8>, Box<dyn Error>> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));

    let nonce = Nonce::from_slice(nonce);

    let cipher = match cipher.encrypt(nonce, data) {
        Ok(cipher) => cipher,
        Err(_) => return Err(Box::<dyn Error>::from("encyption failure")),
    };
    Ok(cipher)
}

fn run_aes_gcm_decryption(
    encryption_data: &[u8],
    key: &[u8; 32],
    nonce: &[u8; 12],
) -> Result<Vec<u8>, Box<dyn Error>> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(nonce);
    match cipher.decrypt(nonce, encryption_data) {
        Ok(data) => Ok(data),
        Err(_) => return Err(Box::<dyn Error>::from("decryption failure")),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde::Deserialize;
    use serde_json::from_reader;
    use std::{fs::File, io::BufReader};

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
}
