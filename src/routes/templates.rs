use crate::search::Entry;
use super::database;
use askama::Template;

#[derive(PartialEq)]
pub enum FormSaved {
    Saved,
    Updated,
    Deleted,
    Nothing
}

#[derive(Debug, Default)]
pub struct FlaggedWord {
    pub word: String,
    pub duplicate: bool,
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate{}

#[derive(Template)]
#[template(path = "view.html")]
pub struct ViewTemplate { 
    pub csv_id: u32,
    pub tl_sentence: String,
    pub nl_sentence: String,
    pub row_order: u32,
    pub csv_row_id: u32,
    pub flashcard_entry: database::FlashcardEntriesEntry,
    pub prev_row_order: Option<u32>,
    pub next_row_order: Option<u32>,
    pub hidden: bool,
    pub words_list: Vec<FlaggedWord>,
    pub was_saved: FormSaved,
}

#[derive(Template)]
#[template(path = "sentence_viewer.html")]
pub struct SentenceViewerTemplate {
   pub word_entry: Entry 
}

#[derive(Template)]
#[template(path = "flashcard_entry_form.html")]
pub struct FileFormTemplate { 
    pub csv_id: u32,
    pub csv_row_id: u32,
    pub row_order: u32,
    pub flashcard_entry: database::FlashcardEntriesEntry,
    pub was_saved: FormSaved,
}
