-- Add migration script here
CREATE TABLE users (
    id UUID PRIMARY KEY,
    phone_number VARCHAR(15) NOT NULL UNIQUE,
    created_at TIMESTAMP NOT NULL DEFAULT now()
);
