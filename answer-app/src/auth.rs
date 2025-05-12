
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};
use lazy_static::lazy_static;
use env_file_reader::read_file;
use std::string::String;
use actix_web::HttpRequest;

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
        let claims = decode::<Claims>(jwt.value(), &DecodingKey::from_secret(&JWT_SECRET), &Validation::default()).unwrap().claims;
        return if claims.exp > OffsetDateTime::now_utc().unix_timestamp() as usize {
            Some(claims)
        } else {
            None
        }
    } else {
        return None;
    } 
}

pub fn get_lang_id(req: HttpRequest) -> u8 {
    return if let Some(jwt) = decode_jwt_from_req(req.clone()) {
        jwt.lngid
    } else if req.connection_info().realip_remote_addr().is_some() {
        1_u8
    } else {
        0_u8
    };
}

pub fn encode_jwt(username: &str, language_id: u8) -> String {
    let expiration = OffsetDateTime::now_utc() + Duration::hours(1);
    let claims = Claims {
        usrnm: username.to_owned(),
        lngid: language_id,
        exp: expiration.unix_timestamp() as usize,
    };
    return encode(&Header::default(), &claims, &EncodingKey::from_secret(&JWT_SECRET)).unwrap();
}

