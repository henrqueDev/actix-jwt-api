-- Your SQL goes here
CREATE TABLE user_types(
    id SERIAL PRIMARY KEY,
    type_name VARCHAR(32) NOT NULL UNIQUE,
    created_at timestamp with time zone,
    updated_at timestamp with time zone,
    deleted_at timestamp with time zone
);

INSERT INTO user_types (id, type_name) VALUES (1, 'admin');
INSERT INTO user_types (id, type_name) VALUES (2, 'stock manager');

CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    email VARCHAR NOT NULL UNIQUE,
    password VARCHAR,
    user_type_id INTEGER NOT NULL REFERENCES user_types(id) ON DELETE CASCADE,
    two_factor_secret TEXT,
    two_factor_recovery_code TEXT,
    two_factor_confirmed_at timestamp with time zone,
    created_at timestamp with time zone,
    updated_at timestamp with time zone,
    deleted_at timestamp with time zone
);

INSERT INTO users (id,name,email,password,user_type_id, created_at,updated_at) VALUES (1, 'Cleverton', 'cleverton@example.com', '$2b$10$B5zK7xNyo2Gkb7qRiQXfKO4qwuWGk3vh0KIQCYBKnawsCFWVeUc.m', 1, '2025-03-30 19:50:48.961943+00','2025-03-30 19:50:48.961943+00'); 