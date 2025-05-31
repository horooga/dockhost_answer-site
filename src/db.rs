use crate::{
    auth::encode_jwt,
    misc::{TEXT, validate},
};
use base64ct::{Base64Bcrypt, Encoding};
use bcrypt::{DEFAULT_COST, hash_with_salt, verify};
use once_cell::sync::Lazy;
use sqlx::{FromRow, PgPool, Postgres, Row, query, query_as};

static SALT_BYTES: Lazy<[u8; 16]> = Lazy::new(|| {
    let bytes = Base64Bcrypt::decode_vec(std::env::var("BCRYPT_SALT").unwrap().as_str()).unwrap();
    bytes[..16].try_into().unwrap()
});

#[derive(FromRow, Debug)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password: String,
}

pub fn hash_password(password: &str) -> String {
    hash_with_salt(password, DEFAULT_COST, *SALT_BYTES)
        .unwrap()
        .to_string()
}

pub async fn login(
    pool: &PgPool,
    username: &String,
    password: &String,
    language_id: u8,
) -> Result<String, String> {
    if let Ok(user) = get_user(pool, username, language_id).await {
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
    pool: &PgPool,
    username: &String,
    password: &String,
    language_id: u8,
) -> Result<(), Vec<String>> {
    if let Err(x) = validate(username, password, language_id).await {
        Err(x)
    } else if let Err(x) = add_user(pool, username, password, language_id).await {
        Err(vec![x])
    } else {
        Ok(())
    }
}

pub async fn add_user(
    pool: &PgPool,
    username: &String,
    password: &String,
    language_id: u8,
) -> Result<(), String> {
    if get_user(&pool, username, language_id).await.is_ok() {
        return Err(TEXT["user_registered"][language_id as usize].to_string());
    }

    if query_as::<Postgres, User>(include_str!("../sql/add_user.sql"))
        .bind(username)
        .bind(hash_password(password))
        .fetch_one(pool)
        .await
        .is_err()
    {
        return Err(
            query_as::<Postgres, User>(include_str!("../sql/add_user.sql"))
                .bind(username)
                .bind(hash_password(password))
                .fetch_one(pool)
                .await
                .unwrap_err()
                .to_string(),
        );
    }

    Err(TEXT["sorry"][language_id as usize].to_string())
}

pub async fn get_user(pool: &PgPool, username: &String, language_id: u8) -> Result<User, String> {
    let query_try = {
        if let Ok(query) = query(include_str!("../sql/get_user.sql"))
            .bind(username)
            .fetch_optional(pool)
            .await
        {
            query
        } else {
            return Err(TEXT["sorry"][language_id as usize].to_string());
        }
    };

    if let Some(query) = query_try {
        Ok(User {
            id: 0,
            username: if let Ok(username) = query.try_get("username") {
                username
            } else {
                return Err(TEXT["user_not_registered"][language_id as usize].to_string());
            },
            password: if let Ok(password) = query.try_get("password") {
                password
            } else {
                return Err(TEXT["user_not_registered"][language_id as usize].to_string());
            },
        })
    } else {
        Err(TEXT["user_not_registered"][language_id as usize].to_string())
    }
}
