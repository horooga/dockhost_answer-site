This is the source of answer-site app container

Env vars:
JWT_SECRET = 32-characters base64 key for HS256 JWT symmetric signing algorithm
BCRYPT_SALT = 22-characters base64 value added to a password before bcrypt hashing
POSTGRES_HOST = postgres container name for dockhost dns
+===answer-app====+  +====postgres=====+
|POSTGRES_USER    |->|POSTGRES_USER    |
|POSTGRES_PASSWORD|->|POSTGRES_PASSWORD|
|POSTGRES_HOST    |  |POSTGRES_DB      |
|BCRYPT_SALT      |  +=================+
|JWT_SECRET       |
+=================+

Postgres container setup:
1) enter "psql -U $POSTGRES_USER -d $POSTGRES_DB" in terminal
2) enter "CREATE TABLE users(
  id SERIAL PRIMARY KEY,
  username VARCHAR(15),
  password VARCHAR(60)
);" query
