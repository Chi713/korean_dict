use crate::search::Entry;
use super::database;
use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate{}

#[derive(Template)]
#[template(path = "view.html")]
pub struct ViewTemplate { 
    pub csv_id: u32,
    pub tl_sentence: String,
    pub nl_sentence: String,
    pub sentence_order: u32,
    pub csv_row_id: u32,
    pub flashcard_entry: database::FlashcardEntriesEntry,
    pub last_csv_row_id: u32,
}

#[derive(Template)]
#[template(path = "sentence_viewer.html")]
pub struct SentenceViewerTemplate {
   pub words_list: Vec<Entry> 
}
