-- Add migration script here

CREATE TABLE IF NOT EXISTS words (
    word_id         INTEGER PRIMARY KEY NOT NULL,
    csv_row_id      INTEGER,
    word            TEXT                NOT NULL,
    is_from_anki    INTEGER DEFAULT 0   NOT NULL,
    is_ignored      INTEGER DEFAULT 0   NOT NULL
);
