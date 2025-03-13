DROP TRIGGER IF EXISTS books_updated_at_trigger ON books;
DROP TRIGGER IF EXISTS items_updated_at_trigger ON items;
DROP TRIGGER IF EXISTS users_updated_at_trigger ON users;

DROP TABLE IF EXISTS returned_checkouts;
DROP TABLE IF EXISTS checkouts;
DROP TABLE IF EXISTS books;
DROP TABLE IF EXISTS items;
DROP TABLE IF EXISTS users;
DROP TABLE IF EXISTS roles;

DROP FUNCTION IF EXISTS set_items_updated_at();
DROP FUNCTION IF EXISTS set_updated_at();
