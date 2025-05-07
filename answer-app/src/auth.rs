
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};
use lazy_static::lazy_static;
use env_file_reader::read_file;
use std::string::String;

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub usrnm: String,
    pub exp: usize,
}

#[derive(Deserialize)]
pub struct UserLogin {
    pub username: String,
    pub password: String,
}

lazy_static! {
    static ref env: std::collections::HashMap<String, String> = read_file(".env").unwrap();
    static ref s: &'static String = &env["secret"];

    pub static ref JWT_SECRET: &'static [u8] = s.as_bytes();
}

pub fn encode_jwt(username: &str) -> String {
    let expiration = OffsetDateTime::now_utc() + Duration::hours(1);
    let claims = Claims {
        usrnm: username.to_owned(),
        exp: expiration.unix_timestamp() as usize,
    };
    return encode(&Header::default(), &claims, &EncodingKey::from_secret(&JWT_SECRET)).unwrap();
}

pub fn decode_jwt(jwt: &String) -> Result<String, &str> {
    let claims = decode::<Claims>(jwt, &DecodingKey::from_secret(&JWT_SECRET), &Validation::default()).unwrap().claims;
    return if claims.exp > OffsetDateTime::now_utc().unix_timestamp() as usize {
        Ok(claims.usrnm)
    } else {
        Err("token is expired")
    }
}

