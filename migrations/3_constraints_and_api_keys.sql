CREATE TABLE IF NOT EXISTS apiKeys (
    "id" UUID PRIMARY KEY,
    encrypted_key TEXT NOT NULL UNIQUE,
    date_created DATE NOT NULL
);

ALTER TABLE tiers
ADD CONSTRAINT tier_name_unique UNIQUE ("name");

ALTER TABLE users
ADD CONSTRAINT user_email_unique UNIQUE (email);