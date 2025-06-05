UPDATE users
SET
  algebra = algebra + CASE WHEN $2 = 'algebra' THEN 1 ELSE 0 END,
  chemistry = chemistry + CASE WHEN $2 = 'chemistry' THEN 1 ELSE 0 END,
  geometry = geometry + CASE WHEN $2 = 'geometry' THEN 1 ELSE 0 END,
  physics = physics + CASE WHEN $2 = 'physics' THEN 1 ELSE 0 END
WHERE username = $1;

