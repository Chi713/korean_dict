use crate::parser::Parser as KrParser;
use crate::parser::{LanguageParser, KhaiiiParser};
use crate::routes::templates::{FileFormTemplate, FlaggedWord};
use crate::search::Session;
use super::database;
use super::templates::{ViewTemplate, SentenceViewerTemplate, FormSaved};
use super::error::RouteError;
use axum::{
    extract::{Query, State, Form},
    Router,
    routing::{get, post, patch, delete},
    response::IntoResponse,
};
use serde::Deserialize;
use sqlx::SqlitePool;

#[derive(Debug, Deserialize)]
struct ViewParams {
    csv_id: u32,
    sentence: Option<u32>
}

pub fn view() -> Router<SqlitePool> {
    async fn handler(
        State(db): State<SqlitePool>,
        Query(params): Query<ViewParams> 
    ) -> Result<impl IntoResponse, RouteError> {
        let csv_id = params.csv_id;
        let sentence_order = params.sentence.unwrap_or(1);
        println!("{:?}", csv_id);

        let sentence = sqlx::query_as::<_,database::CsvRowEntry>(r#"
                SELECT csv_row_id, csv_id, row_order, tag, sq_marker, audio, picture, tl_subs, nl_subs
                FROM csv_row WHERE csv_id = ? AND row_order = ? ;"#)
            .bind(csv_id)
            .bind(sentence_order)
            .fetch_one(&db)
            .await?;

        let flashcard_entry = sqlx::query_as::<_,database::FlashcardEntriesEntry>(r#"
                SELECT flashcard_entries_id, csv_row_id, word, definition
                FROM flashcard_entries WHERE csv_row_id = ?;"#)
            .bind(&sentence.csv_row_id)
            .fetch_optional(&db)
            .await?;
        
        //parse sentence
        let parser = KrParser::new(KhaiiiParser::new());
        println!("sentence: {}", sentence.tl_subs);
        let parsed_sentence = parser.parser
            .parse(&sentence.tl_subs)
            .unwrap();
        println!("parsed sentence: {:?}", parsed_sentence);
        
        let mut filtered_sentence: Vec<FlaggedWord> = vec![];
        for word in parsed_sentence {
            let repeated_words = sqlx::query_as::<_,database::WordEntry>(r#"
                SELECT word_id, csv_row_id, word, is_ignored
                FROM words
                WHERE word = ? ;
            "#)
            .bind(&word)
            .fetch_optional(&db)
            .await?;
            
            let mut flagged_word = FlaggedWord::default();
            flagged_word.word = word;
            if let Some(word) = repeated_words {
                if !word.is_ignored {
                    flagged_word.duplicate = true;
                    filtered_sentence.push(flagged_word);
                }
            } else {
                filtered_sentence.push(flagged_word);
            }
        }

        let flashcard = flashcard_entry.unwrap_or_default();

        let last_csv_row_id = get_last_csv_row_id(csv_id, &db).await?;

        Ok(ViewTemplate {
            csv_id,
            csv_row_id: sentence.csv_row_id,
            tl_sentence: sentence.tl_subs,
            nl_sentence: sentence.nl_subs,
            sentence_order: sentence.row_order,
            flashcard_entry: flashcard,
            last_csv_row_id,
            words_list: filtered_sentence,
            was_saved: FormSaved::Nothing,
        })
    }

    Router::new()
        .route("/view", get(handler))
}

#[derive(Debug, Deserialize)]
struct SentenceViewerParams {
    word: String,
}

pub fn sentence_viewer() -> Router<SqlitePool>{
    async fn handler(
        State(db): State<SqlitePool>,
        Query(params): Query<SentenceViewerParams> 
    ) -> Result<impl IntoResponse, RouteError> {

        println!("\n\ncsv_row_id from query params: {}\n\n",params.word);

        // fix to not instatiate a new session for every request
        let client = Session::new().unwrap();
        let searched_word = client.get(params.word).await.unwrap();
        println!("Searched words list: {:#?}", searched_word);

        Ok(SentenceViewerTemplate {
            word_entry: searched_word
        })
    }

    Router::new()
        .route("/sentence-viewer", get(handler))
}


#[derive(Debug, Deserialize)]
struct FlashCardResponse {
    csv_row_id: u32,
    word: String,
    definition: String,
}

pub fn flashcard_entry_post() -> Router<SqlitePool>{
    async fn handler(
        State(db): State<SqlitePool>,
        Form(data): Form<FlashCardResponse>
    ) -> Result<impl IntoResponse, RouteError> {

        println!("{:?}", &data);
        let flashcard_entry = sqlx::query_as::<_,database::FlashcardEntriesEntry>(r#"
            INSERT INTO flashcard_entries (
                csv_row_id, word, definition
            ) 
            VALUES (?, ?, ? )
            RETURNING * ;"# 
        ).bind(&data.csv_row_id)
            .bind(&data.word)
            .bind(&data.definition)
            .fetch_one(&db)
            .await?;

        let sentence = get_sentence(data.csv_row_id, &db).await?;

        Ok(FileFormTemplate {
            csv_id: sentence.csv_id,
            csv_row_id: sentence.csv_row_id,
            flashcard_entry,
            was_saved: FormSaved::Saved,
        })

    }
    Router::new()
        .route("/view", post(handler))
}

pub fn flashcard_entry_patch() -> Router<SqlitePool>{
    async fn handler(
        State(db): State<SqlitePool>,
        Form(data): Form<FlashCardResponse>
    ) -> Result<impl IntoResponse, RouteError> {

        println!("{:?}", &data);
        let flashcard_entry = sqlx::query_as::<_,database::FlashcardEntriesEntry>(r#"
            UPDATE flashcard_entries 
            SET word = ?, definition = ?
            WHERE csv_row_id = ? 
            RETURNING *;"# 
        ).bind(&data.word)
            .bind(&data.definition)
            .bind(&data.csv_row_id)
            .fetch_one(&db)
            .await?;

        let sentence = get_sentence(data.csv_row_id, &db).await?;

        Ok(FileFormTemplate {
            csv_id: sentence.csv_id,
            csv_row_id: sentence.csv_row_id,
            flashcard_entry,
            was_saved: FormSaved::Updated,
        })
    }
    Router::new()
        .route("/view", patch(handler))
}

pub fn flashcard_entry_delete() -> Router<SqlitePool>{
    async fn handler(
        State(db): State<SqlitePool>,
        Form(data): Form<FlashCardResponse>
    ) -> Result<impl IntoResponse, RouteError> {

        println!("{:?}", &data);
        sqlx::query(r#"
            DELETE FROM flashcard_entries 
            WHERE csv_row_id = ? ;"# 
        ).bind(&data.csv_row_id)
            .execute(&db)
            .await?;

        let sentence = get_sentence(data.csv_row_id, &db).await?;

        let flashcard_entry = database::FlashcardEntriesEntry::default();

        Ok(FileFormTemplate {
            csv_id: sentence.csv_id,
            csv_row_id: sentence.csv_row_id,
            flashcard_entry,
            was_saved: FormSaved::Deleted,
        })
    }
    Router::new()
        .route("/view", delete(handler))
}

async fn get_sentence(csv_row_id: u32, db: &SqlitePool) -> Result<database::CsvRowEntry, RouteError> {

        Ok(sqlx::query_as::<_,database::CsvRowEntry>(r#"
                SELECT csv_row_id, csv_id, row_order, tag, sq_marker, audio, picture, tl_subs, nl_subs
                FROM csv_row WHERE csv_row_id = ? ;"#)
            .bind(csv_row_id)
            .fetch_one(db)
            .await?)
}

async fn get_last_csv_row_id(csv_id: u32, db: &SqlitePool) -> Result<u32, RouteError> {
    let csv_rows = sqlx::query_as::<_,database::CsvLastRowId>(r#"
        SELECT csv_row_id
        FROM csv_row 
        WHERE csv_id = ? ;"#)
    .bind(csv_id)
    .fetch_all(db)
    .await?;

    let csv_last_row_id = csv_rows.into_iter()
        .map(|f| f.csv_row_id)
        .max()
        .unwrap();

    Ok(csv_last_row_id)
}

