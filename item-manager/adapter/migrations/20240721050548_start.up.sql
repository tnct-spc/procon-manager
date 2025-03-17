CREATE OR REPLACE FUNCTION set_updated_at() RETURNS trigger AS '
  BEGIN
    new.updated_at := NOW();
    return new;
  END;
' LANGUAGE 'plpgsql';


CREATE OR REPLACE FUNCTION set_items_updated_at() RETURNS TRIGGER AS '
  BEGIN
      UPDATE items
      SET updated_at = NOW()
      WHERE item_id = NEW.item_id;
      RETURN NEW;
  END;
' LANGUAGE 'plpgsql';


CREATE TABLE IF NOT EXISTS roles (
  role_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  name VARCHAR(255) NOT NULL UNIQUE
);


CREATE TABLE IF NOT EXISTS users (
  user_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  name VARCHAR(255) NOT NULL,
  email VARCHAR(255) NOT NULL UNIQUE,
  password_hash VARCHAR(255) NOT NULL,
  role_id UUID NOT NULL ,
  created_at TIMESTAMP(3) WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
  updated_at TIMESTAMP(3) WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP(3),

  FOREIGN KEY (role_id) REFERENCES roles(role_id)
    ON UPDATE CASCADE
    ON DELETE CASCADE
);

CREATE TRIGGER users_updated_at_trigger
  BEFORE UPDATE ON users FOR EACH ROW
  EXECUTE PROCEDURE set_updated_at();


CREATE TABLE IF NOT EXISTS items (
  item_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  name VARCHAR(255) NOT NULL,
  description VARCHAR(1024) NOT NULL,
  category VARCHAR(255) NOT NULL CHECK (category IN ('general', 'book', 'laptop')),
  created_at TIMESTAMP(3) WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
  updated_at TIMESTAMP(3) WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP(3)
);

CREATE TRIGGER items_updated_at_trigger
  BEFORE UPDATE ON items FOR EACH ROW
  EXECUTE PROCEDURE set_updated_at();


CREATE TABLE IF NOT EXISTS books (
  item_id UUID PRIMARY KEY NOT NULL,
  author VARCHAR(255) NOT NULL,
  isbn VARCHAR(255) NOT NULL,

  FOREIGN KEY (item_id) REFERENCES items(item_id)
    ON UPDATE CASCADE
    ON DELETE CASCADE
);

CREATE TRIGGER books_updated_at_trigger
  BEFORE UPDATE ON books FOR EACH ROW
  EXECUTE PROCEDURE set_items_updated_at();


CREATE TABLE IF NOT EXISTS laptops (
  item_id UUID PRIMARY KEY NOT NULL,
  mac_address macaddr NOT NULL,

  FOREIGN KEY (item_id) REFERENCES items(item_id)
    ON UPDATE CASCADE
    ON DELETE CASCADE
);

CREATE TRIGGER laptops_updated_at_trigger
  BEFORE UPDATE ON laptops FOR EACH ROW
  EXECUTE PROCEDURE set_items_updated_at();


CREATE TABLE IF NOT EXISTS checkouts (
  checkout_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  item_id UUID NOT NULL,
  user_id UUID NOT NULL,
  checked_out_at TIMESTAMP(3) WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP(3),

  FOREIGN KEY (item_id) REFERENCES items(item_id)
    ON UPDATE CASCADE
    ON DELETE CASCADE,

  FOREIGN KEY (user_id) REFERENCES users(user_id)
    ON UPDATE CASCADE
    ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS returned_checkouts (
  checkout_id UUID PRIMARY KEY,
  item_id UUID NOT NULL,
  user_id UUID NOT NULL,
  checked_out_at TIMESTAMP(3) WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
  returned_at TIMESTAMP(3) WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP(3),

  FOREIGN KEY (item_id) REFERENCES items(item_id)
    ON UPDATE CASCADE
    ON DELETE CASCADE,

  FOREIGN KEY (user_id) REFERENCES users(user_id)
    ON UPDATE CASCADE
    ON DELETE CASCADE
);
