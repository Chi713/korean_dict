-- Add migration script here


CREATE TABLE IF NOT EXISTS sentence_words (
    word_id         INTEGER PRIMARY KEY NOT NULL,
    csv_row_id      INTEGER		NOT NULL,
    temp_word       TEXT                NOT NULL,
    FOREIGN KEY (csv_row_id)
	REFERENCES csv_row (csv_row_id)
);
