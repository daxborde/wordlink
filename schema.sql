CREATE TABLE IF NOT EXISTS wordmap (
    id      SERIAL PRIMARY KEY,
    words   VARCHAR(40) NOT NULL,
    link   TEXT NOT NULL
);