ALTER TABLE users 
    -- Add OAuth columns
    ADD COLUMN "provider" TEXT NOT NULL,
    ADD COLUMN provider_user_id TEXT NOT NULL,

    -- Add optional avatar
    ADD COLUMN avatar_url TEXT,

    -- Add timestamp with a default to handle existing rows
    ADD COLUMN created_at TIMESTAMPTZ NOT NULL DEFAULT NOW();

DROP TABLE refresh_tokens;