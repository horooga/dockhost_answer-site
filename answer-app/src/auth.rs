
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};
use lazy_static::lazy_static;
use dotenv::dotenv;

lazy_static! {
    dotenv().ok();

    static ref s: String = std::env::var("secret").unwrap();
    pub static ref JWT_SECRET: &'static [u8] = s.as_bytes();
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub usrnm: String,
    pub exp: usize,
}

#[derive(Deserialize)]
pub struct UserLogin {
    pub username: String,
    pub password: String,
}
