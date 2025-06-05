SELECT username,
       algebra,
       chemistry,
       geometry,
       physics,
       (algebra + chemistry + geometry + physics) AS answers_sum
FROM users
ORDER BY answers_sum DESC
LIMIT 10;
