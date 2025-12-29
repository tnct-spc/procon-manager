ALTER TABLE checkouts
    ADD CONSTRAINT checkouts_item_id_key UNIQUE (item_id);
