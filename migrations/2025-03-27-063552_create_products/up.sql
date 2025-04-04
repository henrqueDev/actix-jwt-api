-- Your SQL goes here
CREATE TABLE products (
    id SERIAL PRIMARY KEY,
    SKU VARCHAR(255) UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    price REAL NOT NULL,
    weight REAL NOT NULL,
    dimension_height REAL NOT NULL,
    dimension_width REAL NOT NULL, -- Dimensões em metros
    dimension_depth REAL NOT NULL,
    product_category_id INTEGER NOT NULL REFERENCES product_categories(id) ON DELETE CASCADE,
    created_at timestamp with time zone,
    updated_at timestamp with time zone,
    deleted_at timestamp with time zone
);

CREATE UNIQUE INDEX products_sku ON products(SKU);