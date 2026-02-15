CREATE TABLE IF NOT EXISTS tiers (
    "id" UUID PRIMARY KEY,
    "name" TEXT NOT NULL,
    max_tracks INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS users (
    "id" UUID PRIMARY KEY,
    "name" TEXT NOT NULL,
    email TEXT NOT NULL UNIQUE,
    tier_uuid UUID NOT NULL,

    CONSTRAINT fk_tier 
        FOREIGN KEY (tier_uuid) 
        REFERENCES tiers ("id")
        ON DELETE RESTRICT
);