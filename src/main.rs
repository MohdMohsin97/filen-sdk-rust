
mod http;
use crate::http::Client;
use serde::Serialize;
use http::HttpMethod as method;

#[derive(Debug, Serialize)]
struct InfoRequest {
    email: String,

}

fn main() {

    let client = Client::new(String::from("CHIIIT"));
    let req = client
            .request(method::GET,String::from("/v3/auth/info"),Some(InfoRequest{email: String::from("Rireje3001@heweek.com")})).unwrap();
    println!("{:#?}",req);   
}