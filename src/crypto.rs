use ring::digest::{self, digest};
use ring::pbkdf2::{self, Algorithm};
use std::error::Error;
use std::num::NonZeroU32;

pub fn derive_key_from_password(
    password: String,
    salt: String,
    hash: Algorithm,
    iterations: u32,
    bit_length: usize,
) -> Result<String, Box<dyn Error>> {
    let drive_key = run_pbkdf2(password, salt, hash, iterations, bit_length)?;

    let _first_half = &drive_key[0..(drive_key.len() / 2)];
    let second_half = &drive_key[(drive_key.len() / 2)..];

    let key = run_sha512(second_half.to_owned());

    Ok(key)
}

fn run_pbkdf2(
    password: String,
    salt: String,
    hash: Algorithm,
    iterations: u32,
    bit_length: usize,
) -> Result<String, Box<dyn Error>> {
    let mut key = vec![0u8; bit_length / 8];

    pbkdf2::derive(
        hash,
        NonZeroU32::new(iterations).expect("Invalid Iterations"),
        salt.as_bytes(),
        password.as_bytes(),
        &mut key,
    );

    let drive_key = hex::encode(key);

    Ok(drive_key)
}

fn run_sha512(data: String) -> String {
    let hex = digest(&digest::SHA512, data.as_bytes());

    hex::encode(hex)
}

#[cfg(test)]
mod test {
    use super::*;
    use pbkdf2::PBKDF2_HMAC_SHA512;
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

        let auth_key = derive_key_from_password(
            test_info.password,
            test_info.salt,
            PBKDF2_HMAC_SHA512,
            200_000,
            512,
        )
        .unwrap();

        assert_eq!(test_info.auth_key, auth_key);
    }
}
