-- Add up migration script here

CREATE TABLE IF NOT EXISTS athlete (
    id BIGINT PRIMARY KEY,
    username TEXT,
    created_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ,
    max_speed REAL,
    max_power REAL,
    max_cadence REAL,
    total_distance REAL,
    total_ascension REAL,
    total_energy REAL
);

CREATE TABLE IF NOT EXISTS session (
    athlete_id BIGINT PRIMARY KEY REFERENCES athlete (id),
    uuid UUID UNIQUE,
    refresh_token TEXT,
    access_token TEXT,
    access_expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ
);
