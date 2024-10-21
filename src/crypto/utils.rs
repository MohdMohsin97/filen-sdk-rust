use aes_gcm::aead::rand_core::RngCore;
use aes_gcm::aead::{Aead, OsRng};
use aes_gcm::{Aes256Gcm, Key, KeyInit, Nonce};
use ring::digest::{self, digest};
use ring::pbkdf2::{self, Algorithm};
use std::error::Error;
use std::num::NonZeroU32;

pub fn run_sha512(data: String) -> String {
    let hex = digest(&digest::SHA512, data.as_bytes());

    hex::encode(hex)
}

pub fn generate_random_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}

pub fn run_aes_gcm_encryprion(
    data: &[u8],
    key: &[u8; 32],
    nonce: &super::Nonce,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));

    let nonce = Nonce::from_slice(nonce.as_slice());

    cipher
        .encrypt(nonce, data)
        .map_err(|_| Box::<dyn Error>::from("encyption failure"))
}

pub fn run_aes_gcm_decryption(
    encryption_data: &[u8],
    key: &[u8; 32],
    nonce: &super::Nonce,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(nonce.as_slice());
    match cipher.decrypt(nonce, encryption_data) {
        Ok(data) => Ok(data),
        Err(_) => return Err(Box::<dyn Error>::from("decryption failure")),
    }
}

pub fn run_pbkdf2(
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

pub fn vec_u8_to_string(vec: Vec<u8>) -> String {
    vec.iter().map(ToString::to_string).collect()
}
