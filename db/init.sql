CREATE TABLE items (
    item_id SERIAL PRIMARY KEY,
    table_no INT NOT NULL CHECK (table_no >= 1 AND table_no <= 100),
    item VARCHAR(255) NOT NULL,
    time INT
);
