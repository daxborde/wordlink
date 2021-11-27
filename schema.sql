-- This file is for developer reference, and is not used by the 
-- program. There is a string in the create_wordmap_table function
-- that is identical to the one below, which is used for table 
-- creation.
CREATE TABLE IF NOT EXISTS wordmap (
    id      SERIAL PRIMARY KEY,
    words   VARCHAR(40) NOT NULL,
    link    TEXT NOT NULL
);