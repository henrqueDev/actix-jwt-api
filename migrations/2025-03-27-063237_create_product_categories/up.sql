-- Your SQL goes here

CREATE TABLE product_categories (
    id SERIAL PRIMARY KEY,
    name VARCHAR(128) UNIQUE NOT NULL,
    created_at timestamp with time zone,
    updated_at timestamp with time zone,
    deleted_at timestamp with time zone
);

INSERT INTO product_categories (id, name) VALUES (1, 'Eletrônicos');
INSERT INTO product_categories (id, name) VALUES (2, 'Roupas');