use std::sync::Arc;
use crate::parser::Parser as KrParser;
use crate::parser::{LanguageParser, KhaiiiParser};
use crate::search::Session;
use super::database;
use axum::{
    extract::{Query, State, Form},
    Router,
    http::StatusCode,
    routing::{get, post, patch},
    Extension,
    response::{Html, IntoResponse},
};
use serde::Deserialize;
use tera::{Tera, Context};
use sqlx::{Sqlite, SqlitePool, QueryBuilder};

const BIND_LIMIT: usize = 65535;


#[derive(Debug, Deserialize)]
struct SentenceViewerParams {
    csv_id: u32,
    sentence: Option<u32>
}

pub fn view() -> Router<SqlitePool> {
    async fn handler(
        State(db): State<SqlitePool>,
        Extension(templates): Extension<Arc<Tera>>,
        Query(params): Query<SentenceViewerParams> 
    ) -> Result<impl IntoResponse, (StatusCode, String)> {
        let csv_id = params.csv_id;
        let sentence_order = match params.sentence {
            Some(sentence_order) => sentence_order,
            None => 1
        };
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

        let flashcard = match flashcard_entry {
            Some(val) => val,
            None => database::FlashcardEntriesEntry::default()
        };

        let mut context = Context::new();
        context.insert("csv_id", &csv_id);
        context.insert("tl_sentence", &sentence.tl_subs);
        context.insert("nl_sentence", &sentence.nl_subs);
        context.insert("sentence_order", &sentence.row_order);
        context.insert("csv_row_id", &sentence.csv_row_id);
        context.insert("flashcard_entry", &flashcard);
        Ok(Html(templates.render("view.html", &context).unwrap()))
    }

    Router::new()
        .route("/view", get(handler))
}

pub fn sentence_viewer() -> Router<SqlitePool>{
    async fn handler(
        State(db): State<SqlitePool>,
        Extension(templates): Extension<Arc<Tera>>,
        Query(params): Query<SentenceViewerParams> 
    ) -> Result<impl IntoResponse, (StatusCode, String)> {
        let csv_id = params.csv_id;
        let sentence_order = match params.sentence {
            Some(sentence_order) => sentence_order,
            None => 1
        };
        println!("{:?}", csv_id);

        let sentence_db_response = sqlx::query_as::<_,database::CsvRowEntry>(r#"
                SELECT csv_row_id, csv_id, row_order, tag, sq_marker, audio, picture, tl_subs, nl_subs
                FROM csv_row WHERE csv_id = ? AND row_order = ? ;"#)
            .bind(csv_id)
            .bind(sentence_order)
            .fetch_one(&db)
            .await
            .unwrap();

        let tl_sentence = sentence_db_response.tl_subs;
        // let sentence_order = sentence_db_response.row_order;
        let csv_row_id = sentence_db_response.csv_row_id;

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

        let mut context = Context::new();
        context.insert("words_list", &searched_words_list);
        Ok(Html(templates.render("sentence_viewer.html", &context).unwrap()))
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
        Extension(templates): Extension<Arc<Tera>>,
        Form(data): Form<FlashCardResponse>
    ) -> Result<impl IntoResponse, (StatusCode, String)> {

        println!("{:?}", &data);
        let flashcard_entry = sqlx::query_as::<_,database::FlashcardEntriesEntry>(r#"
            INSERT INTO flashcard_entries (
                csv_row_id, word, definition
            ) VALUES (?, ?, ? );"# 
        ).bind(&data.csv_row_id)
            .bind(&data.word)
            .bind(&data.definition)
            .fetch_one(&db)
            .await
            .unwrap();

        let sentence = sqlx::query_as::<_,database::CsvRowEntry>(r#"
                SELECT csv_row_id, csv_id, row_order, tag, sq_marker, audio, picture, tl_subs, nl_subs
                FROM csv_row WHERE csv_row_id = ?;"#)
            .bind(&flashcard_entry.csv_row_id)
            .fetch_one(&db)
            .await
            .unwrap();

        let mut context = Context::new();
        context.insert("csv_id", &sentence.csv_id);
        context.insert("tl_sentence", &sentence.tl_subs);
        context.insert("nl_sentence", &sentence.nl_subs);
        context.insert("sentence_order", &sentence.row_order);
        context.insert("csv_row_id", &sentence.csv_row_id);
        context.insert("flashcard_entry", &flashcard_entry);
        Ok(Html(templates.render("view.html", &context).unwrap()))
    }
    Router::new()
        .route("/view", post(handler))
}

pub fn flashcard_entry_patch() -> Router<SqlitePool>{
    async fn handler(
        State(db): State<SqlitePool>,
        Extension(templates): Extension<Arc<Tera>>,
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

        let sentence = sqlx::query_as::<_,database::CsvRowEntry>(r#"
                SELECT csv_row_id, csv_id, row_order, tag, sq_marker, audio, picture, tl_subs, nl_subs
                FROM csv_row WHERE csv_row_id = ?;"#)
            .bind(&flashcard_entry.csv_row_id)
            .fetch_one(&db)
            .await
            .unwrap();

        let mut context = Context::new();
        context.insert("csv_id", &sentence.csv_id);
        context.insert("tl_sentence", &sentence.tl_subs);
        context.insert("nl_sentence", &sentence.nl_subs);
        context.insert("sentence_order", &sentence.row_order);
        context.insert("csv_row_id", &sentence.csv_row_id);
        context.insert("flashcard_entry", &flashcard_entry);
        Ok(Html(templates.render("view.html", &context).unwrap()))
    }
    Router::new()
        .route("/view", patch(handler))
}
