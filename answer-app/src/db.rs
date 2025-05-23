use crate::{
    auth::encode_jwt,
    misc::{ENV, TEXT, validate},
};
use base64ct::{Base64Bcrypt, Encoding};
use bcrypt::{DEFAULT_COST, hash_with_salt, verify};
use deadpool_postgres::Client;
use once_cell::sync::Lazy;

static SALT_BYTES: Lazy<[u8; 16]> = Lazy::new(|| {
    let bytes = Base64Bcrypt::decode_vec(&ENV["BCRYPT_SALT"]).unwrap();
    bytes[..16].try_into().unwrap()
});

pub struct User {
    pub username: String,
    pub password: String,
}

pub fn hash_password(password: &str) -> String {
    hash_with_salt(password, DEFAULT_COST, *SALT_BYTES)
        .unwrap()
        .to_string()
}

pub async fn login(
    client: &Client,
    username: &String,
    password: &String,
    language_id: u8,
) -> Result<String, String> {
    if let Ok(user) = get_user(client, username, language_id).await {
        if verify(password, user.password.as_str()).unwrap() {
            Ok(encode_jwt(username, 0_u8))
        } else {
            Err(TEXT["login_wrong"][language_id as usize].to_string())
        }
    } else {
        Err(TEXT["user_not_registered"][language_id as usize].to_string())
    }
}

pub async fn register(
    client: &Client,
    username: &String,
    password: &String,
    language_id: u8,
) -> Result<(), Vec<String>> {
    if let Err(x) = validate(username, password, language_id).await {
        Err(x)
    } else if let Err(x) = add_user(client, username, password, language_id).await {
        Err(vec![x])
    } else {
        Ok(())
    }
}

pub async fn add_user(
    client: &Client,
    username: &String,
    password: &String,
    language_id: u8,
) -> Result<(), String> {
    let _stmt = include_str!("../sql/add_user.sql");
    let stmt = client.prepare(_stmt).await.unwrap();

    if get_user(client, username, language_id).await.is_ok() {
        return Err(TEXT["user_registered"][language_id as usize].to_string());
    }

    if client
        .query(&stmt, &[&username, &hash_password(password)])
        .await
        .is_ok()
    {
        return Ok(());
    }

    Err(TEXT["sorry"][language_id as usize].to_string())
}

pub async fn get_user(client: &Client, username: &String, language_id: u8) -> Result<User, String> {
    let _stmt = include_str!("../sql/get_user.sql");
    let _stmt = _stmt.replace("$username", username.as_str());
    let stmt = client.prepare(&_stmt).await.unwrap();

    if let Ok(x) = client.query(&stmt, &[]).await {
        if !x.is_empty() {
            Ok(User {
                username: x[0].get(1),
                password: x[0].get(2),
            })
        } else {
            Err(TEXT["user_not_registered"][language_id as usize].to_string())
        }
    } else {
        Err(TEXT["sorry, try again later"][language_id as usize].to_string())
    }
}
