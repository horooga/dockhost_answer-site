use crate::{
    auth::encode_jwt,
    misc::{TEXT, validate},
};
use bcrypt::{DEFAULT_COST, hash, verify};
use serde::Serialize;
use sqlx::{FromRow, PgPool, Row, query};

#[derive(FromRow, Debug, Serialize)]
pub struct User {
    pub username: String,
    pub password: String,
    pub algebra: i32,
    pub chemistry: i32,
    pub geometry: i32,
    pub physics: i32,
}

pub fn hash_password(password: &str) -> String {
    hash(password, DEFAULT_COST).unwrap().to_string()
}

pub async fn register(
    pool: &PgPool,
    username: &str,
    password: &str,
    language_id: u8,
) -> Result<(), Vec<String>> {
    if let Err(x) = validate(&username.to_string(), &password.to_string(), language_id).await {
        Err(x)
    } else if let Err(x) = add_user(pool, username, password, language_id).await {
        Err(vec![x])
    } else {
        Ok(())
    }
}

pub async fn login(
    pool: &PgPool,
    username: &str,
    password: &str,
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

pub async fn add_user(
    pool: &PgPool,
    username: &str,
    password: &str,
    language_id: u8,
) -> Result<(), String> {
    if get_user(pool, username, language_id).await.is_ok() {
        return Err(TEXT["user_registered"][language_id as usize].to_string());
    }

    if query(include_str!("../sql/add_user.sql"))
        .bind(username)
        .bind(hash_password(password))
        .execute(pool)
        .await
        .is_ok()
    {
        return Ok(());
    }

    Err(TEXT["sorry"][language_id as usize].to_string())
}

pub async fn get_user(pool: &PgPool, username: &str, language_id: u8) -> Result<User, String> {
    let query = {
        if let Ok(query) = query(include_str!("../sql/get_user.sql"))
            .bind(username)
            .fetch_one(pool)
            .await
        {
            query
        } else {
            return Err(TEXT["user_not_registered"][language_id as usize].to_string());
        }
    };

    Ok(User {
        username: query.try_get("username").unwrap(),
        password: query.try_get("password").unwrap(),
        algebra: query.try_get("algebra").unwrap(),
        chemistry: query.try_get("chemistry").unwrap(),
        geometry: query.try_get("geometry").unwrap(),
        physics: query.try_get("physics").unwrap(),
    })
}

pub async fn get_top(pool: &PgPool) -> Vec<User> {
    query(include_str!("../sql/get_top.sql"))
        .fetch_all(pool)
        .await
        .unwrap()
        .into_iter()
        .map(|r| User {
            username: r.try_get("username").unwrap(),
            password: "".to_string(),
            algebra: r.try_get("algebra").unwrap(),
            chemistry: r.try_get("chemistry").unwrap(),
            geometry: r.try_get("geometry").unwrap(),
            physics: r.try_get("physics").unwrap(),
        })
        .collect()
}

pub async fn update_user(pool: &PgPool, username: &str, topic: &str) {
    let _ = query(include_str!("../sql/update_user.sql"))
        .bind(username)
        .bind(topic)
        .execute(pool)
        .await;
}
