CREATE TABLE devices (
    id BIGSERIAL PRIMARY KEY,
    address TEXT UNIQUE NOT NULL,
    username TEXT NOT NULL,
    encrypted_password BYTEA NOT NULL,
    password_nonce BYTEA NOT NULL
);
