CREATE TABLE racing_events (
    "id" UUID PRIMARY KEY,
    event_name TEXT NOT NULL,
    created_at TIMESTAMP
);

CREATE TABLE user_events (
    "id" UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    racing_id UUID NOT NULL REFERENCES racing_events(id) ON DELETE CASCADE
);