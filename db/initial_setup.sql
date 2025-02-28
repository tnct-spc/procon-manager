INSERT INTO
    roles (name)
VALUES
    ('Admin'),
    ('User')
ON CONFLICT DO NOTHING;

INSERT INTO
    users (name, email, password_hash, role_id)
SELECT
    'Admin',
    '',
    '$2b$12$rwekkrZ97L5oSYs.QiPYoeYDYImqWHo7xvL2wyrhJ/6tJGRDCgXHa',
    role_id
FROM
    roles
WHERE
    name LIKE 'Admin';
