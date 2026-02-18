CREATE TABLE refresh_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash TEXT NOT NULL, -- HASH PLS
    expires_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP DEFAULT now()
);