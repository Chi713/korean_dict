-- Add migration script here

CREATE TABLE IF NOT EXISTS flashcard_entries (
    flashcard_entries_id    INTEGER PRIMARY KEY NOT NULL, 
    csv_row_id		    INTEGER		NOT NULL, 
    word		    TEXT                NOT NULL,
    definition		    TEXT                NOT NULL,
    FOREIGN KEY (csv_row_id)
	REFERENCES csv_row (csv_row_id)
);
