-- Generate general items
INSERT INTO items (item_id, name, description, category, created_at, updated_at)
SELECT
    gen_random_uuid(),
    'item' || LPAD(i::text, 3, '0'),
    'description' || LPAD(i::text, 3, '0'),
    'general',
    now() - (i || ' seconds')::interval,
    now() - (i || ' seconds')::interval
FROM generate_series(1, 30) i;

-- Generate books
WITH inserted_books AS (
    INSERT INTO items (item_id, name, description, category, created_at, updated_at)
    SELECT
        gen_random_uuid(),
        'book' || LPAD(i::text, 3, '0'),
        'book description' || LPAD(i::text, 3, '0'),
        'book',
        now() - ((i + 30) || ' seconds')::interval,
        now() - ((i + 30) || ' seconds')::interval
    FROM generate_series(1, 10) i
    RETURNING item_id
)
INSERT INTO books (item_id, author, isbn)
SELECT
    item_id,
    'author' || LPAD(id::text, 3, '0'),
    LPAD(id::text, 13, '0')
FROM (SELECT item_id, ROW_NUMBER() OVER () as id FROM inserted_books) sub;

-- Generate laptops
WITH inserted_laptops AS (
    INSERT INTO items (item_id, name, description, category, created_at, updated_at)
    SELECT
        gen_random_uuid(),
        'laptop' || LPAD(i::text, 3, '0'),
        'laptop description' || LPAD(i::text, 3, '0'),
        'laptop',
        now() - ((i + 50) || ' seconds')::interval,
        now() - ((i + 50) || ' seconds')::interval
    FROM generate_series(1, 10) i
    RETURNING item_id
)
INSERT INTO laptops (item_id, mac_address)
SELECT
    item_id,
    ('00:00:00:00:00:' || LPAD(FLOOR(RANDOM() * 100)::text, 2, '0'))::macaddr
FROM inserted_laptops;
