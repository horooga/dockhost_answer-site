use deadpool_postgres::Client;
use crate::{misc::TEXT};

pub struct User {
    pub username: String,
    pub password: String,
}

pub async fn add_user(client: &Client, username: &String, password: &String, language_id: u8) {
    let _stmt = include_str!("../sql/add_user.sql");
    let stmt = client.prepare(&_stmt).await.unwrap();

    client
        .query(
            &stmt,
            &[
                username,
                password,
                &(language_id as i8),
            ],
        )
        .await;
}

pub async fn get_user(client: &Client, username: &String, language_id: u8) -> Result<User, String> {
    let _stmt = include_str!("../sql/get_user.sql");
    let _stmt = _stmt.replace("$username", username);
    let stmt = client.prepare(&_stmt).await.unwrap();

    return match client
        .query(
            &stmt,
            &[],
        )
        .await {
        Ok(x) => 
            if !x.is_empty() {
               Ok(User {
                   username: x[0].get(0),
                   password: x[0].get(1),
               })
            } else {
               Err(TEXT["user_registered"][language_id as usize].to_string())
            },
        Err(_) => Err(TEXT["sorry"][language_id as usize].to_string()),
    }
}
