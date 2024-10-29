CREATE TABLE items (
    item_id SERIAL PRIMARY KEY,
    table_no INT NOT NULL,
    item VARCHAR(255) NOT NULL,
    time INT
);
