use deadpool_postgres::Client;

pub struct User {
    pub username: String,
    pub password: String,
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

pub async fn get_user(client: &Client, username: &String) -> Result<User, String> {
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
               Err("User is not registered".to_string())
            },
        Err(_) => Err("Sorry, try again later".to_string()),
    }
}
