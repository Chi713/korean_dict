-- Add migration script here

CREATE TABLE IF NOT EXISTS subtitles (
    subtitle_id     INTEGER PRIMARY KEY NOT NULL,
    srt_id          INTEGER             NOT NULL,
    subtitle_order  INTEGER             NOT NULL,
    subtitle_start  VARCHAR(15)         NOT NULL,
    subtitle_end    VARCHAR(15)         NOT NULL,
    subtitle_text   VARCHAR(8000)       NOT NULL,
    FOREIGN KEY (srt_id)
        REFERENCES srt (srt_id)
);
