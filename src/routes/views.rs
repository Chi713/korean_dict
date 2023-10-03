use crate::parser::Parser as KrParser;
use crate::parser::{LanguageParser, KhaiiiParser};
use crate::search::Session;
use super::database;
use super::templates::{ViewTemplate, SentenceViewerTemplate};
use axum::{
    extract::{Query, State, Form},
    Router,
    http::StatusCode,
    routing::{get, post, patch, delete},
    response::IntoResponse,
};
use serde::Deserialize;
use sqlx::{Sqlite, SqlitePool, QueryBuilder};
use std::error::Error;

const BIND_LIMIT: usize = 65535;

#[derive(Debug, Deserialize)]
struct ViewParams {
    csv_id: u32,
    sentence: Option<u32>
}

pub fn view() -> Router<SqlitePool> {
    async fn handler(
        State(db): State<SqlitePool>,
        Query(params): Query<ViewParams> 
    ) -> Result<impl IntoResponse, (StatusCode, String)> {
        let csv_id = params.csv_id;
        let sentence_order = params.sentence.unwrap_or(1);
        println!("{:?}", csv_id);

        let sentence = sqlx::query_as::<_,database::CsvRowEntry>(r#"
                SELECT csv_row_id, csv_id, row_order, tag, sq_marker, audio, picture, tl_subs, nl_subs
                FROM csv_row WHERE csv_id = ? AND row_order = ? ;"#)
            .bind(csv_id)
            .bind(sentence_order)
            .fetch_one(&db)
            .await
            .unwrap();

        let flashcard_entry = sqlx::query_as::<_,database::FlashcardEntriesEntry>(r#"
                SELECT flashcard_entries_id, csv_row_id, word, definition
                FROM flashcard_entries WHERE csv_row_id = ?;"#)
            .bind(&sentence.csv_row_id)
            .fetch_optional(&db)
            .await
            .unwrap();

        let flashcard = flashcard_entry.unwrap_or_default();

        let last_csv_row_id = get_last_csv_row_id(csv_id, &db).await.unwrap();

        Ok(ViewTemplate {
            csv_id,
            csv_row_id: sentence.csv_row_id,
            tl_sentence: sentence.tl_subs,
            nl_sentence: sentence.nl_subs,
            sentence_order: sentence.row_order,
            flashcard_entry: flashcard,
            last_csv_row_id,
        })
    }

    Router::new()
        .route("/view", get(handler))
}

#[derive(Debug, Deserialize)]
struct SentenceViewerParams {
    csv_row_id: u32,
}

pub fn sentence_viewer() -> Router<SqlitePool>{
    async fn handler(
        State(db): State<SqlitePool>,
        Query(params): Query<SentenceViewerParams> 
    ) -> Result<impl IntoResponse, (StatusCode, String)> {

        println!("\n\ncsv_row_id from query params: {}\n\n",params.csv_row_id);

        let sentence = get_sentence(params.csv_row_id, &db).await.unwrap();

        let tl_sentence = sentence.tl_subs;
        let csv_row_id = sentence.csv_row_id;

        // parse sentence
        let client = Session::new().unwrap();
        let parser = KrParser::new(KhaiiiParser::new());
        println!("sentence: {tl_sentence}");
        let mut parsed_sentence = parser.parser.parse(&tl_sentence).unwrap();
        println!("parsed sentence: {:?}", parsed_sentence);
        
        // filter out words to be ignored
        // must fix to not insert every request
        let mut sentence_word_query_builder: QueryBuilder<Sqlite> = QueryBuilder::new(
            "INSERT INTO sentence_words (
                csv_row_id, temp_word
            ) " 
        ); 

        sentence_word_query_builder.push_values(parsed_sentence.iter().take(BIND_LIMIT/2), |mut b, word| {
            b.push_bind(csv_row_id)
                .push_bind(word);
        });

        let sentence_word_query = sentence_word_query_builder.build();
        sentence_word_query.execute(&db).await.unwrap();

        let ignored_words = sqlx::query_as::<_,database::WordEntry>(r#"
                SELECT words.word_id, sentence_words.csv_row_id, words.word, words.is_ignored
                FROM words
                LEFT JOIN sentence_words
                ON words.word = sentence_words.temp_word AND words.is_ignored = 1;
            "#)
            .bind(csv_row_id)
            .fetch_all(&db)
            .await
            .unwrap();

        println!("dup words: {:?}", ignored_words);

        sqlx::query(r#"DELETE FROM sentence_words"#).execute(&db).await.unwrap();

        // find better way to remove words
        // refactor to make dup deleting from the table an sql query
        parsed_sentence.retain(|word| {
            let mut flag = true;
            for ignored_word in ignored_words.iter() {
                if word == &ignored_word.word && ignored_word.is_ignored {
                    flag = false
                }
            }
            flag
        });

        // get dictionary entry from parsed sentence words
        let searched_words_list = client.get_list(parsed_sentence).await.unwrap();
        println!("Searched words list: {:#?}", searched_words_list);

        Ok(SentenceViewerTemplate {
            words_list: searched_words_list
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
    ) -> Result<impl IntoResponse, (StatusCode, String)> {

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
            .await
            .unwrap();

        let sentence = get_sentence(data.csv_row_id, &db).await.unwrap();

        let last_csv_row_id = get_last_csv_row_id(sentence.csv_id, &db).await.unwrap();

        Ok(ViewTemplate {
            csv_id: sentence.csv_id,
            csv_row_id: sentence.csv_row_id,
            tl_sentence: sentence.tl_subs,
            nl_sentence: sentence.nl_subs,
            sentence_order: sentence.row_order,
            flashcard_entry,
            last_csv_row_id,
        })

    }
    Router::new()
        .route("/view", post(handler))
}

pub fn flashcard_entry_patch() -> Router<SqlitePool>{
    async fn handler(
        State(db): State<SqlitePool>,
        Form(data): Form<FlashCardResponse>
    ) -> Result<impl IntoResponse, (StatusCode, String)> {

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
            .await
            .unwrap();

        let sentence = get_sentence(data.csv_row_id, &db).await.unwrap();

        let last_csv_row_id = get_last_csv_row_id(sentence.csv_id, &db).await.unwrap();

        Ok(ViewTemplate {
            csv_id: sentence.csv_id,
            csv_row_id: sentence.csv_row_id,
            tl_sentence: sentence.tl_subs,
            nl_sentence: sentence.nl_subs,
            sentence_order: sentence.row_order,
            flashcard_entry,
            last_csv_row_id,
        })
    }
    Router::new()
        .route("/view", patch(handler))
}

pub fn flashcard_entry_delete() -> Router<SqlitePool>{
    async fn handler(
        State(db): State<SqlitePool>,
        Form(data): Form<FlashCardResponse>
    ) -> Result<impl IntoResponse, (StatusCode, String)> {

        println!("{:?}", &data);
        let delete_query = sqlx::query(r#"
            DELETE FROM flashcard_entries 
            WHERE csv_row_id = ? ;"# 
        ).bind(&data.csv_row_id)
            .execute(&db)
            .await;
        match delete_query {
            Ok(_) => (),
            Err(e) => return Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }

        let sentence = get_sentence(data.csv_row_id, &db).await.unwrap();

        let flashcard_entry = database::FlashcardEntriesEntry::default();

        let last_csv_row_id = get_last_csv_row_id(sentence.csv_id, &db).await.unwrap();

        Ok(ViewTemplate {
            csv_id: sentence.csv_id,
            csv_row_id: sentence.csv_row_id,
            tl_sentence: sentence.tl_subs,
            nl_sentence: sentence.nl_subs,
            sentence_order: sentence.row_order,
            flashcard_entry,
            last_csv_row_id,
        })
    }
    Router::new()
        .route("/view", delete(handler))
}

async fn get_sentence(csv_row_id: u32, db: &SqlitePool) -> Result<database::CsvRowEntry, Box<dyn Error>> {

        Ok(sqlx::query_as::<_,database::CsvRowEntry>(r#"
                SELECT csv_row_id, csv_id, row_order, tag, sq_marker, audio, picture, tl_subs, nl_subs
                FROM csv_row WHERE csv_row_id = ? ;"#)
            .bind(csv_row_id)
            .fetch_one(db)
            .await?)
}

async fn get_last_csv_row_id(csv_id: u32, db: &SqlitePool) -> Result<u32, Box<dyn Error>> {
    // let csv_last_row = sqlx::query_as::<_,database::CsvLastRowId>(r#"
    //     SELECT LAST_VALUE (csv_row_id) OVER (
    //         ORDER BY csv_row_id 
    //         RANGE BETWEEN UNBOUNDED PRECEDING AND 
    //         UNBOUNDED FOLLOWING
    //     )
    //     FROM csv_row 
    //     WHERE csv_id= ? ;"#)
    // .bind(csv_id)
    // .fetch_one(db)
    // .await
    // .unwrap();

    let csv_rows = sqlx::query_as::<_,database::CsvLastRowId>(r#"
        SELECT csv_row_id
        FROM csv_row 
        WHERE csv_id = ? ;"#)
    .bind(csv_id)
    .fetch_all(db)
    .await?;

    let csv_last_row_id = csv_rows.into_iter().map(|f| f.csv_row_id).max().unwrap();

    Ok(csv_last_row_id)
}
