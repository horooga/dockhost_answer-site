use actix_web::HttpRequest;
use env_file_reader::read_file;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::string::String;
use time::{Duration, OffsetDateTime};

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub usrnm: String,
    pub lngid: u8,
    pub exp: usize,
}

#[derive(Deserialize)]
pub struct UserLogin {
    pub username: String,
    pub password: String,
}

lazy_static! {
    pub static ref ENV: std::collections::HashMap<String, String> = read_file(".env").unwrap();
    static ref s: &'static String = &ENV["JWT_SECRET"];
    pub static ref JWT_SECRET: &'static [u8] = s.as_bytes();
}

pub fn decode_jwt_from_req(req: HttpRequest) -> Option<Claims> {
    if let Some(jwt) = req.cookie("token") {
        let claims = decode::<Claims>(
            jwt.value(),
            &DecodingKey::from_secret(&JWT_SECRET),
            &Validation::default(),
        )
        .unwrap()
        .claims;
        if claims.exp > OffsetDateTime::now_utc().unix_timestamp() as usize {
            Some(claims)
        } else {
            None
        }
    } else {
        None
    }
}

pub fn encode_jwt(username: &str, language_id: u8) -> String {
    let expiration = OffsetDateTime::now_utc() + Duration::hours(1);
    let claims = Claims {
        usrnm: username.to_owned(),
        lngid: language_id,
        exp: expiration.unix_timestamp() as usize,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(&JWT_SECRET),
    )
    .unwrap()
}

pub fn get_lang_id(req: HttpRequest) -> u8 {
    if let Some(jwt) = decode_jwt_from_req(req.clone()) {
        jwt.lngid
    } else {
        0_u8
    }
}
