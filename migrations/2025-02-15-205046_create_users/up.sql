-- Your SQL goes here
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    email VARCHAR NOT NULL UNIQUE,
    password VARCHAR NOT NULL,
    two_factor_secret TEXT,
    two_factor_recovery_code TEXT,
    two_factor_confirmed_at timestamp with time zone,
    created_at timestamp with time zone,
    updated_at timestamp with time zone,
    deleted_at timestamp with time zone
);

INSERT INTO users (id,name,email,password,created_at,updated_at) VALUES (1, 'Cleverton', 'cleverton@example.com', '$2b$10$B5zK7xNyo2Gkb7qRiQXfKO4qwuWGk3vh0KIQCYBKnawsCFWVeUc.m', '2025-03-30 19:50:48.961943+00','2025-03-30 19:50:48.961943+00'); 