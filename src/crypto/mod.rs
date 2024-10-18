mod utils;

use aes_gcm::aead::{rand_core::RngCore, OsRng};
use base64::prelude::*;
use ring::pbkdf2::{Algorithm, PBKDF2_HMAC_SHA512};
use std::{error::Error, fmt::Display, marker::PhantomData, str::FromStr};
use utils::{run_aes_gcm_decryption, run_aes_gcm_encryprion, run_pbkdf2, run_sha512};

const METADATA_IDENTIFIER: &str = "002";

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

struct Nonce([u8; 12]);

impl Nonce {
    fn from_slice(slice: &[u8]) -> Self {
        let mut nonce = [0u8; 12];
        nonce.copy_from_slice(slice);
        Self(nonce)
    }

    fn as_slice(&self) -> &[u8] {
        &self.0
    }
}

impl Display for Nonce {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            // SAFETY: Nonce is randomly generated and hence it may have invalid utf8 chars which is
            // valid in this sense
            unsafe { String::from_utf8_unchecked(self.0.to_vec()) }
        )
    }
}

#[derive(Clone, Debug)]
struct Key([u8; 32]);

impl Key {
    fn from_slice(slice: &[u8]) -> Self {
        let mut key = [0u8; 32];
        key.copy_from_slice(slice);
        Self(key)
    }

    fn as_slice(&self) -> &[u8; 32] {
        &self.0
    }
}

struct MetadataCrypto {
    key: Key,
    nonce: Nonce,
    metadata: Vec<u8>,
}

impl MetadataCrypto {
    fn new(metadata: impl AsRef<[u8]>, key: &Key) -> Result<Self, ()> {
        let nonce = {
            let mut nonce = [0u8; 12];
            OsRng.fill_bytes(&mut nonce);
            Nonce::from_slice(&nonce)
        };

        let encrypted_metadata =
            run_aes_gcm_encryprion(metadata.as_ref(), key.as_slice(), &nonce).map_err(|_| ())?;

        Ok(Self {
            key: key.clone(),
            nonce,
            metadata: encrypted_metadata,
        })
    }

    pub fn new_with_key(key: Key) -> Self {
        Self {
            key,
            nonce: Nonce([0u8; 12]),
            metadata: Vec::new(),
        }
    }

    // pub decrypt_with_key(key: &Key) -> Vec<u8> {}
}

impl FromStr for MetadataCrypto {
    type Err = ();

    fn from_str(metadata: &str) -> Result<Self, Self::Err> {
        todo!("Moseen bhai");
    }
}

impl Display for MetadataCrypto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            METADATA_IDENTIFIER,
            self.nonce.to_string(),
            BASE64_STANDARD.encode(&self.metadata)
        )
    }
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

        let metadata = MetadataCrypto::new(
            "Hello this is a test".to_owned(),
            &Key::from_slice(&generate_random_key()),
        )
        .unwrap();

        let encrypted = metadata.to_string();

        let decrypt_data = decrypt_metadata(encrypted, generate_random_key()).unwrap();

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
