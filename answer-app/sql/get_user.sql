SELECT username, password, algebra, chemistry, geometry, physics
FROM users
WHERE username = $1;
