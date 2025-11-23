-- Your SQL goes here
CREATE TABLE users (
   id INT8 PRIMARY KEY,
   username VARCHAR(30) NOT NULL UNIQUE,
   email TEXT NOT NULL UNIQUE,
   password_hash TEXT NOT NULL,
   balance MONEY NOT NULL DEFAULT 0,
   claim_expires_timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
