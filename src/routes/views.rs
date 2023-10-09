use crate::parser::Parser as LangParser;
use crate::parser::{LanguageParser, KhaiiiParser};
use crate::routes::templates::{FileFormTemplate, FlaggedWord};
use crate::search::Session;
use super::database;
use super::templates::{ViewTemplate, SentenceViewerTemplate, FormSaved};
use super::error::RouteError;
use anyhow::Context;
use serde::Deserialize;
use sqlx::SqlitePool;
use axum::{
    extract::{Query, State, Form},
    Router,
    routing::{get, post, patch, delete},
    response::IntoResponse,
};

#[derive(Debug, Deserialize)]
struct ViewParams {
    csv_id: u32,
    row_order: Option<u32>
}

pub fn view() -> Router<SqlitePool> {
    async fn handler(
        State(db): State<SqlitePool>,
        Query(params): Query<ViewParams> 
    ) -> Result<impl IntoResponse, RouteError> {
        let csv_id = params.csv_id;

        // this should double check that a given row order is in bounds
        let csv_row = match params.row_order {
            Some(row_order) => {
                get_csv_row_by_row_order(csv_id, row_order, &db).await?
            },
            None => {
                // unwrap is happening because flashcard file could be empty, handle properly later
                get_next_flashcard(csv_id, 0, &db).await?.unwrap()
            }
        };

        let flashcard_entry = sqlx::query_as::<_, database::FlashcardEntriesEntry>(r#"
                SELECT flashcard_entries_id, csv_row_id, word, definition
                FROM flashcard_entries WHERE csv_row_id = ?;"#)
            .bind(&csv_row.csv_row_id)
            .fetch_optional(&db)
            .await?
            .unwrap_or_default();

        // parse sentence
        let parser = LangParser::new(KhaiiiParser::new());
        let parsed_sentence = parser.parser.parse(&csv_row.tl_subs)?;
        let filtered_sentence = filter_sentence(parsed_sentence, &db).await?;

        let prev_row_order = get_prev_flashcard(csv_id, csv_row.row_order, &db)
            .await
            .context("error getting prev row")?
            .map(|r| r.row_order);
        let next_row_order = get_next_flashcard(csv_id, csv_row.row_order, &db)
            .await
            .context("error getting next row")?
            .map(|r| r.row_order);

        Ok(ViewTemplate {
            csv_id,
            csv_row_id: csv_row.csv_row_id,
            tl_sentence: csv_row.tl_subs,
            nl_sentence: csv_row.nl_subs,
            row_order: csv_row.row_order,
            flashcard_entry,
            prev_row_order,
            next_row_order,
            hidden: filtered_sentence.is_empty(),
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
        State(_db): State<SqlitePool>,
        Query(params): Query<SentenceViewerParams> 
    ) -> Result<impl IntoResponse, RouteError> {

        // fix to not instatiate a new session for every request
        let client = Session::new()?;
        let searched_word = client.get(params.word).await?;

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
    row_order: u32,
    word: String,
    definition: String,
}

pub fn flashcard_entry_post() -> Router<SqlitePool>{
    async fn handler(
        State(db): State<SqlitePool>,
        Form(data): Form<FlashCardResponse>
    ) -> Result<impl IntoResponse, RouteError> {

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

        let csv_row = get_csv_row_by_id(data.csv_row_id, &db).await?;

        Ok(FileFormTemplate {
            csv_id: csv_row.csv_id,
            csv_row_id: csv_row.csv_row_id,
            row_order: data.row_order,
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

        let csv_row = get_csv_row_by_id(data.csv_row_id, &db).await?;

        Ok(FileFormTemplate {
            csv_id: csv_row.csv_id,
            csv_row_id: csv_row.csv_row_id,
            row_order: data.row_order,
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

        sqlx::query(r#"
            DELETE FROM flashcard_entries 
            WHERE csv_row_id = ? ;"# 
        ).bind(&data.csv_row_id)
            .execute(&db)
            .await?;

        let csv_row = get_csv_row_by_id(data.csv_row_id, &db).await?;

        let flashcard_entry = database::FlashcardEntriesEntry::default();

        Ok(FileFormTemplate {
            csv_id: csv_row.csv_id,
            csv_row_id: csv_row.csv_row_id,
            row_order: data.row_order,
            flashcard_entry,
            was_saved: FormSaved::Deleted,
        })
    }
    Router::new()
        .route("/view", delete(handler))
}

async fn get_csv_row_by_id(csv_row_id: u32, db: &SqlitePool) -> Result<database::CsvRowEntry, RouteError> {
    Ok(sqlx::query_as::<_,database::CsvRowEntry>(r#"
            SELECT csv_row_id, csv_id, row_order, tag, sq_marker, audio, picture, tl_subs, nl_subs
            FROM csv_row WHERE csv_row_id = ? ;"#)
        .bind(csv_row_id)
        .fetch_one(db)
        .await?)
}

async fn get_csv_row_by_row_order(
    csv_id: u32, row_order: u32, db: &SqlitePool
) -> Result<database::CsvRowEntry, RouteError> {
    Ok(sqlx::query_as::<_,database::CsvRowEntry>(r#"
            SELECT csv_row_id, csv_id, row_order, tag, sq_marker, audio, picture, tl_subs, nl_subs
            FROM csv_row WHERE csv_id = ? AND row_order = ? ;"#)
        .bind(csv_id)
        .bind(row_order)
        .fetch_one(db)
        .await?)

}

async fn get_last_csv_row_order(csv_id: u32, db: &SqlitePool) -> Result<u32, RouteError> {
    let csv_rows = sqlx::query_as::<_,database::CsvLastRowId>(r#"
        SELECT row_order
        FROM csv_row 
        WHERE csv_id = ? ;"#)
    .bind(csv_id)
    .fetch_all(db)
    .await?;

    let csv_last_row_order = csv_rows.into_iter()
        .map(|r| r.row_order)
        .max();

    if let Some(row_order) = csv_last_row_order {
        Ok(row_order)
    } else {
        Err(RouteError::LastCsvRowError)
    }
}

async fn filter_sentence(sentence: Vec<String>, db: &SqlitePool ) -> Result<Vec<FlaggedWord>, RouteError> {
        let mut filtered_sentence: Vec<FlaggedWord> = vec![];
        for word in sentence {
            let repeated_words = sqlx::query_as::<_,database::WordEntry>(r#"
                SELECT word_id, csv_row_id, word, is_ignored
                FROM words
                WHERE word = ? ;
            "#)
            .bind(&word)
            .fetch_optional(db)
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
    Ok(filtered_sentence)
}

async fn get_next_flashcard(csv_id: u32, mut row_order: u32, db: &SqlitePool) -> Result<Option<database::CsvRowEntry>, RouteError> {
    let last_row_order = get_last_csv_row_order(csv_id, db).await?; 
    if row_order == last_row_order { return Ok(None); }

    let mut csv_row: database::CsvRowEntry;
    loop {
        row_order += 1;
        csv_row = get_csv_row_by_row_order(csv_id, row_order, db).await?;
       
        let parser = LangParser::new(KhaiiiParser::new());
        let parsed_sentence = parser.parser.parse(&csv_row.tl_subs)?;
        let filtered_sentence = filter_sentence(parsed_sentence, db).await?;

        if !filtered_sentence.is_empty() {
            return Ok(Some(csv_row))
        }
        if csv_row.row_order >= last_row_order {
            return Ok(None);
        }
    }
}

async fn get_prev_flashcard(csv_id: u32, mut row_order: u32, db: &SqlitePool) -> Result<Option<database::CsvRowEntry>, RouteError> {
    if row_order == 1 { return Ok(None); }

    let mut csv_row: database::CsvRowEntry;
    loop {
        row_order -= 1;
        csv_row = get_csv_row_by_row_order(csv_id, row_order, db).await?;

        let parser = LangParser::new(KhaiiiParser::new());
        let parsed_sentence = parser.parser.parse(&csv_row.tl_subs)?;
        let filtered_sentence = filter_sentence(parsed_sentence, db).await?;

        if !filtered_sentence.is_empty() {
            return Ok(Some(csv_row))
        }
        if csv_row.row_order <= 1 {
            return Ok(None);
        }
    }
}
