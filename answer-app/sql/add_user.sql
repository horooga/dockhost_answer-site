INSERT INTO testing.users(username, password, language)
VALUES ($1, $2, $3, $4)
RETURNING username, password, language;
