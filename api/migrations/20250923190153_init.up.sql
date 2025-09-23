-- Add up migration script here

CREATE TABLE IF NOT EXISTS athlete (
    id SERIAL PRIMARY KEY,
    username TEXT,
    created_at DATE,
    updated_at DATE,
    max_speed REAL,
    max_power REAL,
    max_cadence REAL,
    total_distance REAL,
    total_ascension REAL,
    total_energy REAL
);

CREATE TABLE IF NOT EXISTS session (
    uuid UUID PRIMARY KEY,
    athlete_id INT REFERENCES athlete (id),
    refresh_token TEXT,
    access_token TEXT,
    access_expires_at DATE,
    created_at DATE
);
