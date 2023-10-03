use crate::csv_parser;
use super::database;
use axum::{
    extract::{Multipart, State, Query},
    Router,
    http::{StatusCode, header},
    routing::{get, post},
    response::{Redirect, IntoResponse},
};
use sqlx::{Sqlite, SqlitePool, QueryBuilder};
use serde::Deserialize;

const BIND_LIMIT: usize = 65535;

pub fn process_file_data() -> Router<SqlitePool> {
    async fn handler(
        State(db): State<SqlitePool>, 
        mut multipart: Multipart
    ) -> Result<Redirect, (StatusCode, String)> {
        let mut csv_id = 0;
        while let Some(field) = multipart.next_field().await.unwrap() {
            println!("multipart interating");
            let name = field.name().unwrap().to_string();
            if &name == "retimer" {
                let on: bool = field.text().await.unwrap() == "on";
                println!("retiming choosen: {on}");
            } else if &name == "data" {
                let file_name = field.file_name().unwrap().to_string();
                let content_type = field.content_type().unwrap().to_string();
                let data = field.text().await.unwrap();

                println!(
                    "File: `{}` (`{}`: `{}`) is {} bytes", name, file_name, content_type, data.len()
                );

                // csv parsing
                let csv_res = csv_parser::csv_parse(&data).unwrap();

                csv_id = sqlx::query(r#"INSERT INTO csv (file_name) VALUES (?);"#)
                    .bind(file_name)
                    .execute(&db)
                    .await
                    .unwrap()
                    .last_insert_rowid();

                let mut subtitle_query_builder: QueryBuilder<Sqlite> = QueryBuilder::new(
                    "INSERT INTO csv_row (
                        csv_id, row_order, tag, sq_marker, audio, picture, tl_subs, nl_subs
                    ) " 
                ); 

                // used zip because enumerate returns usize
                let csv_data_iter = (1u32..).zip(csv_res.data.iter());
                
                subtitle_query_builder.push_values(csv_data_iter.take(BIND_LIMIT/8), |mut b, (row_order, sub)| {
                    b.push_bind(csv_id)
                        .push_bind(row_order)
                        .push_bind(sub.get("tag"))
                        .push_bind(sub.get("sq_marker"))
                        .push_bind(sub.get("audio"))
                        .push_bind(sub.get("picture"))
                        .push_bind(sub.get("tl_subs"))
                        .push_bind(sub.get("nl_subs"));
                });

                let subtitle_query = subtitle_query_builder.build();
                subtitle_query.execute(&db).await.unwrap();
            }
        }

        let sentence_viewer_uri = format!("/view?csv_id={}", csv_id);
        Ok(Redirect::to(&sentence_viewer_uri))
    }

    Router::new()
        .route("/", post(handler))
}

#[derive(Debug, Deserialize)]
struct ProduceCsvFileParams {
    csv_id: u32,
}

pub fn produce_csv_file() -> Router<SqlitePool> {
    async fn handler(
        State(db): State<SqlitePool>, 
        Query(params): Query<ProduceCsvFileParams>,
    ) -> Result<impl IntoResponse, (StatusCode, String)> {
        let csv_rows: Vec<database::CsvFileEntry> = sqlx::query_as::<_,database::CsvFileEntry>(r#"
            SELECT row.csv_row_id, row.csv_id, row.tag, row.sq_marker, row.audio, row.picture, row.tl_subs, row.nl_subs, card.word, card.definition 
            FROM csv_row as row
            INNER JOIN flashcard_entries as card
            ON row.csv_id = ? AND row.csv_row_id = card.csv_row_id ;
        "#).bind(params.csv_id)
            .fetch_all(&db)
            .await
            .unwrap();

        let file = csv_parser::build_csv_file(csv_rows).unwrap();
        println!("\n\n csv file: {}\n\n", file);

        let headers = [
            (header::CONTENT_TYPE, r#"text/csv; charset=utf-8"#),
            (header::CONTENT_DISPOSITION, r#"attachment; filename="data.csv"#),
        ];
        Ok((headers, file))
        
    }
    Router::new()
        .route("/get-file", get(handler))
}
