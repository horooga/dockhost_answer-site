Security:
    - jwt in cookies authentication middleware
    - bcrypt password hashing

Env vars:
    - JWT_SECRET - 32-characters base64 key for HS256 JWT symmetric signing algorithm
    - POSTGRES_HOST - postgres service name for dockhost dns

    +===answer-app====+  +====postgres=====+
    |POSTGRES_USER    |<-|POSTGRES_USER    |
    |POSTGRES_PASSWORD|<-|POSTGRES_PASSWORD|
    |POSTGRES_HOST    |  |POSTGRES_DB      |
    |JWT_SECRET       |  +=================+
    +=================+

Postgres container setup:
    1) enter "psql -U $POSTGRES_USER -d $POSTGRES_DB" in terminal
    2) enter "CREATE TABLE users(
        id SERIAL PRIMARY KEY,
        username VARCHAR(15),
        password VARCHAR(60),
        algebra INT DEFAULT 0,
        chemistry INT DEFAULT 0,
        geometry INT DEFAULT 0,
        physics INT DEFAULT 0
    );" query

App container setup:
    1) specify container's env vars and 8000 port
