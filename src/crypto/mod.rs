mod utils;
use base64::prelude::*;
use ring::pbkdf2::{Algorithm, PBKDF2_HMAC_SHA512};
use std::{error::Error, fmt::Display};
use utils::{
    generate_random_key, run_aes_gcm_decryption, run_aes_gcm_encryprion, run_pbkdf2, run_sha512,
};

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

struct Nonce(Vec<u8>);

impl Nonce {
    fn from_slice(slice: &str) -> Self {
        Self(slice.as_bytes().to_vec())
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
            String::from_utf8(self.0.to_owned()).unwrap()
        )
    }
}

#[derive(Clone, Debug)]
struct Key(Vec<u8>);

impl Key {
    fn from_slice(slice: &str) -> Self {
        Self(slice.as_bytes().to_vec())
    }

    fn as_slice(&self) -> &[u8] {
        &self.0
    }
}

struct MetadataEncrypto {
    key: Key,
    nonce: Nonce,
    metadata: Vec<u8>,
}

impl MetadataEncrypto {
    fn new(metadata: impl AsRef<[u8]>, key: &Key) -> Result<Self, ()> {
        let nonce = Nonce::from_slice(&generate_random_key(12));

        let encrypted_metadata =
            run_aes_gcm_encryprion(metadata.as_ref(), key, &nonce).map_err(|_| ())?;

        Ok(Self {
            key: key.clone(),
            nonce,
            metadata: encrypted_metadata,
        })
    }
}

impl Display for MetadataEncrypto {
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

struct MetadataDecrypto {
    metadata: String,
    key: Key,
}

impl MetadataDecrypto {
    fn new(metadata: &str, key: &Key) -> Result<Self, ()> {
        let nonce_str = &metadata[3..15];

        let nonce = Nonce::from_slice(nonce_str);

        let encrypted = BASE64_STANDARD.decode(&metadata[15..]).map_err(|_| ())?;

        let result = run_aes_gcm_decryption(&encrypted, key, &nonce)
            .map_err(|_| println!("decryption failed"))?;

        let result_str = String::from_utf8(result).map_err(|_| ())?;

        Ok(MetadataDecrypto {
            metadata: result_str,
            key: key.clone(),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use easy_hasher::easy_hasher::{sha1, sha512};
    use serde::Deserialize;
    use serde_json::from_reader;
    use std::{fs::File, io::BufReader};
    use utils::generate_random_key;

    #[derive(Deserialize, Debug)]
    struct TestInfo {
        email: String,
        password: String,
        salt: String,
        auth_key: String,
    }

    pub fn hash_fn<S: Into<String>>(value: S) -> String {
        sha1(&sha512(&value.into()).to_hex_string()).to_hex_string()
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
        let nonce = Nonce::from_slice(&generate_random_key(12));

        let key = Key::from_slice(&generate_random_key(32));

        let encrypt_data = run_aes_gcm_encryprion(data, &key, &nonce).unwrap();

        let decrypt_data = run_aes_gcm_decryption(&encrypt_data, &key, &nonce).unwrap();

        assert_eq!(decrypt_data, data);
    }

    #[test]
    fn test_metadata_encrypt_decrypt() {
        let metadata = String::from("This is test metadata");

        let key = Key::from_slice("abcdefghijklmnopqrstuvwxyzABCDEF");

        let encypt_metadata = MetadataEncrypto::new(metadata.to_owned(), &key).unwrap();

        let decrypt_metadata = MetadataDecrypto::new(&encypt_metadata.to_string(), &key).unwrap();

        assert_eq!(metadata, decrypt_metadata.metadata);
    }
}
