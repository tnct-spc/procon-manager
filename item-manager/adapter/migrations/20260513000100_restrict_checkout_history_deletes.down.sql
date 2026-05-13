ALTER TABLE checkouts
    DROP CONSTRAINT IF EXISTS checkouts_item_id_fkey,
    ADD CONSTRAINT checkouts_item_id_fkey
        FOREIGN KEY (item_id) REFERENCES items(item_id)
        ON UPDATE CASCADE
        ON DELETE CASCADE;

ALTER TABLE checkouts
    DROP CONSTRAINT IF EXISTS checkouts_user_id_fkey,
    ADD CONSTRAINT checkouts_user_id_fkey
        FOREIGN KEY (user_id) REFERENCES users(user_id)
        ON UPDATE CASCADE
        ON DELETE CASCADE;

ALTER TABLE returned_checkouts
    DROP CONSTRAINT IF EXISTS returned_checkouts_item_id_fkey,
    ADD CONSTRAINT returned_checkouts_item_id_fkey
        FOREIGN KEY (item_id) REFERENCES items(item_id)
        ON UPDATE CASCADE
        ON DELETE CASCADE;

ALTER TABLE returned_checkouts
    DROP CONSTRAINT IF EXISTS returned_checkouts_user_id_fkey,
    ADD CONSTRAINT returned_checkouts_user_id_fkey
        FOREIGN KEY (user_id) REFERENCES users(user_id)
        ON UPDATE CASCADE
        ON DELETE CASCADE;
