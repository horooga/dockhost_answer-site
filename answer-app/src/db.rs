use deadpool_postgres::Client;
use serde::{Deserialize, Serialize};
use tokio_pg_mapper_derive::PostgresMapper;

#[derive(Deserialize, PostgresMapper, Serialize)]
#[pg_mapper(table = "users")]
pub struct User {
    pub username: String,
    pub password: String,
    pub language: String,
}

pub async fn add_user(client: &Client, username: &String, password: &String, language: &String) {
    let _stmt = include_str!("../sql/add_user.sql");
    let stmt = client.prepare(&_stmt).await.unwrap();

    client
        .query(
            &stmt,
            &[
                &username,
                &password,
                &language,
            ],
        )
        .await;
}

pub async fn get_user(client: &Client, username: &String) -> Result<String, String> {
    let _stmt = include_str!("../sql/get_user.sql");
    let stmt = client.prepare(&_stmt).await.unwrap();

    return match client
        .query(
            &stmt,
            &[],
        )
        .await {
        Ok(x) => Ok(x[0].columns()[0].name().to_string()),
        Err(_) => Err("User is not registered!".to_string()),
    }
}
