-- Your SQL goes here
CREATE TABLE products (
    id SERIAL PRIMARY KEY,
    SKU VARCHAR(255) UNIQUE NOT NULL,
    name VARCHAR(255),
    description TEXT,
    price MONEY NOT NULL,
    weight REAL NOT NULL,
    dimension_height DECIMAL(5,3),
    dimension_width DECIMAL(5,3), -- Dimensões em metros
    dimension_depth DECIMAL(5,3),
    category_id INTEGER NOT NULL REFERENCES product_categories(id)
);