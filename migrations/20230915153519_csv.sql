-- Add migration script here

CREATE TABLE IF NOT EXISTS csv (
    csv_id	INTEGER PRIMARY KEY 	NOT NULL, 
    file_name	VARCHAR(250)		NOT NULL
);
