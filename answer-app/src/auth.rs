
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub usrnm: String,
    #[serde(with = "jwt_numeric_date")]
    pub exp: usize,
}

#[derive(Deserialize)]
pub struct UserLogin {
    pub username: String,
    pub password: String,
}
