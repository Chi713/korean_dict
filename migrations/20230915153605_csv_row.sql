-- Add migration script here

CREATE TABLE IF NOT EXISTS csv_row (
    csv_row_id	INTEGER PRIMARY KEY NOT NULL, 
    csv_id	INTEGER             NOT NULL, 
    row_order	INTEGER             NOT NULL, 
    tag         TEXT                NOT NULL,
    sq_marker   TEXT                NOT NULL,
    audio       TEXT                NOT NULL,
    picture     TEXT                NOT NULL,                
    tl_subs     TEXT                NOT NULL,
    nl_subs     TEXT                NOT NULL,
    FOREIGN KEY (csv_id)
	REFERENCES csv (csv_id)
);
