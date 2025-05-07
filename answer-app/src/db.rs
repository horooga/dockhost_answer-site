use deadpool_postgres::Client;
use tokio_pg_mapper::FromTokioPostgresRow;
use serde::{Deserialize, Serialize};
use tokio_pg_mapper_derive::PostgresMapper;

#[derive(Deserialize, PostgresMapper, Serialize)]
#[pg_mapper(table = "users")]
pub struct User {
    pub username: String,
    pub password: String,
    pub language: String,
}

pub async fn add_user(client: &Client, user: User) {
    let _stmt = include_str!("../sql/add_user.sql");
    let stmt = client.prepare(&_stmt).await.unwrap();

    client
        .query(
            &stmt,
            &[
                &user.username,
                &user.password,
                &user.language,
            ],
        )
        .await;
}
