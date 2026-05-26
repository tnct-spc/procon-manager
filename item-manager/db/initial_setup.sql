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
    'admin@example.com',
    '$2b$12$ZIaGw7tfyojuJgbd2jHt1OK/RksKOrxeIamRoWafYVocfNcXY9v/K',
    role_id
FROM
    roles
WHERE
    name LIKE 'Admin';
