mod crypto;
mod http;
use crate::http::ApiResponse as ApiResponse;
use crate::http::Client;
use serde::{Serialize, Deserialize};
use http::HttpMethod as method;
use crate::crypto::{derive_key_from_password, DeriveKeyFromPasswordParams};
use std::fs::File;
use std::io::BufReader;
use serde_json::from_reader;



#[derive(Debug, Serialize)]
struct InfoRequest {
    email: String,

}

#[derive(Deserialize, Debug)]
struct AuthInfo {
    email: String,
    password: String,
    salt: String,
    id: u64,
    auth_key: String,
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
    fn test_derive_key_from_password() {
        let file_path = "test_inputs.json";
        let test_struct_info = read_auth_info_from_file(file_path).unwrap();
        let params = DeriveKeyFromPasswordParams {
            password: test_struct_info.password,
            salt: test_struct_info.salt,
            iterations: 200000,
            hash: "sha512".to_string(),
            bit_length: 512,
            return_hex: false,
            environment: "node".to_string(),
        };

        let key = derive_key_from_password(params).unwrap();
        assert_eq!(key, test_struct_info.auth_key);
    }

    //test 2
    #[test]
    fn test_auto_info_request() {
        let expected_response = ApiResponse {
            status: true,
            message: "Auth info fetched.".to_string(),
            code: "auth_info_fetched".to_string(),
            data: Some(serde_json::json!({
                "authVersion": 2,
                "email": "rireje3001@heweek.com",
                "id": 92022189,
                "salt": "LwHVPQ3lPccMrhWnX86dlpdoop46i2pwsMTUs9zs6mQugKjUYJlxxndPUKI3Cnbgpo8Kvq7M7aWUXwqtrF9WhwwCnOuunMHT2nP7kYSEJc2peNDeUl0WZCmBKwKbJpCOA0sB2w2oba24UPelBwJVlJBrNzWk8hQB40LchPJS15zKfcXFSG9XZGhmrHHB0B6JYxu3v3AwdkehgxkoK8SxvjH6p61jKCv2M6HyI9IsZJtlSCpIS0BBeBSSkscm6rx2"
            })),
        };
        let client = Client::new(String::from("CHIIIT"));
        let actual_response = client
            .request(method::POST,String::from("/v3/auth/info"),Some(InfoRequest{email: String::from("rireje3001@heweek.com")})).unwrap();
        assert_eq!(actual_response.status, expected_response.status);
        assert_eq!(actual_response.message, expected_response.message);
        assert_eq!(actual_response.code, expected_response.code);
        assert_eq!(actual_response.data, expected_response.data);
    }
}


