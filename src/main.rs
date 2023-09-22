use clap::Parser as ClapParser;
use sqlx::{QueryBuilder, Pool};
use std::str::FromStr;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::sync::Arc;
use std::env;
use korean_dict::csv_parser;
use korean_dict::parser::Parser as KrParser;
use korean_dict::parser::{LanguageParser, KhaiiiParser};
use korean_dict::search::Session;
use axum::{
    extract::{Multipart, Query, State},
    Router,
    http::StatusCode,
    routing::{get, post},
    Extension,
    response::{Html, IntoResponse, Redirect},
};
use serde::Deserialize;
use dotenvy::dotenv;
use log::info;
use tower_http::services::ServeDir;
use tera::{Tera, Context};
use sqlx::{FromRow, migrate::MigrateDatabase ,Sqlite, SqlitePool};

// Setup the command line interface with clap.
#[derive(ClapParser, Debug)]
#[clap(name = "server", about = "A server for our wasm project!")]
struct Opt {
    /// set the log level
    #[clap(short = 'l', long = "log", default_value = "debug")]
    log_level: String,
    /// set the listen addr
    #[clap(short = 'a', long = "addr", default_value = "127.0.0.1")]
    addr: String,

    /// set the listen port
    #[clap(short = 'p', long = "port", default_value = "8080")]
    port: u16,

    /// set the directory where static files are to be found
    #[clap(long = "static-dir", default_value = "./dist")]
    static_dir: String,
}

const DB_URL: &str = "sqlite://database.db";
const BIND_LIMIT: usize = 65535;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    env_logger::init();
    info!("such information");

    // set options from command line
    let opt = Opt::parse();
    let sock_addr = SocketAddr::from((
        IpAddr::from_str(opt.addr.as_str()).unwrap_or(IpAddr::V6(Ipv6Addr::LOCALHOST)),
        opt.port,
    ));

    // set database connection
    if !Sqlite::database_exists(DB_URL).await.unwrap_or(false) {
        println!("Creating database {}", DB_URL);
        match Sqlite::create_database(DB_URL).await {
            Ok(_) => info!("Create db success"),
            Err(error) => panic!("error: {}", error),
        }
    } else {
        println!("Database already exists");
    }
    let db = SqlitePool::connect(DB_URL).await.unwrap();

    // run database migrations
    let _migrations = std::path::Path::new(&crate_dir).join("./migrations");
    let migration_results = sqlx::migrate!()
        .run(&db)
        .await;

    match migration_results {
        Ok(_) => {info!("migrations ran sucessfully");},
        Err(err) => {panic!("Error: {}", err);}
    }

    // bring in html templates
    let tera = Tera::new("templates/**/*.html").unwrap();
    let app = Router::new()
        .merge(view())
        .merge(process_file_data())
        .merge(sentence_viewer())
        .with_state(db)
        .merge(index())
        .nest_service("/favicon.ico", ServeDir::new("public/favicon.ico"))
        .layer(Extension(Arc::new(tera)));

    info!("listening on http://{}", sock_addr);

    axum::Server::bind(&sock_addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn index() -> Router {
    async fn handler(
        Extension(templates): Extension<Arc<Tera>>,
    ) -> impl IntoResponse {
        let context = Context::new();
        Html(templates.render("index.html", &context).unwrap())
    }
    Router::new().route("/", get(|template: Extension<Arc<Tera>>| handler(template)))
}

#[derive(Clone, FromRow, Debug)]
struct CsvEntry {
    csv_id:i64,
    file_name: String
}

#[derive(Clone, FromRow, Debug)]
struct CsvRowEntry {
    csv_row_id: u32,
    csv_id: u32,
    row_order: u32,
    tag: String,
    audio: String,
    picture: String,
    tl_subs: String,
    nl_subs: String
}


fn process_file_data() -> Router<SqlitePool> {
    async fn handler(State(db): State<SqlitePool>, mut multipart: Multipart) -> Result<Redirect, (StatusCode, String)> {
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

                let srt_db_data = sqlx::query_as::<_,CsvRowEntry>(r#"
                    SELECT csv_row_id, csv_id, row_order, tag, sq_marker, audio, picture, tl_subs, nl_subs
                    FROM csv_row WHERE csv_id = ? ;"#)
                    .bind(csv_id)
                    .fetch_all(&db)
                    .await
                    .unwrap();
                    
                println!("{:?}", srt_db_data);
            }
        }

        let sentence_viewer_uri = format!("/view?csv_id={}", csv_id);
        Ok(Redirect::to(&sentence_viewer_uri))
    }

    Router::new()
        .route("/api/fileData", post(|db: State<Pool<Sqlite>>, 
            multipart: Multipart| handler(db, multipart)))
}

#[derive(Debug, Deserialize)]
struct SentenceViewerParams {
    csv_id: u32,
    sentence: Option<u32>
}

#[derive(Clone, FromRow, Debug)]
struct WordEntry {
    word_id: u32,
    csv_row_id: u32,
    word: String,
    is_ignored: bool
}


fn view() -> Router<Pool<Sqlite>> {
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

        let sentence_db_response = sqlx::query_as::<_,CsvRowEntry>(r#"
                SELECT csv_row_id, csv_id, row_order, tag, sq_marker, audio, picture, tl_subs, nl_subs
                FROM csv_row WHERE csv_id = ? AND row_order = ? ;"#)
            .bind(csv_id)
            .bind(sentence_order)
            .fetch_one(&db)
            .await
            .unwrap();

        let tl_sentence = sentence_db_response.tl_subs;
        let nl_sentence = sentence_db_response.nl_subs;
        let sentence_order = sentence_db_response.row_order;
        // let csv_row_id = sentence_db_response.csv_row_id;

        let mut context = Context::new();
        context.insert("csv_id", &csv_id);
        context.insert("tl_sentence", &tl_sentence);
        context.insert("nl_sentence", &nl_sentence);
        context.insert("sentence_order", &sentence_order);
        Ok(Html(templates.render("view.html", &context).unwrap()))
    }

    Router::new()
        .route("/view", get(|db: State<Pool<Sqlite>>, 
            template: Extension<Arc<Tera>>, 
            query: Query<SentenceViewerParams>| handler(db,template, query)))
}

fn sentence_viewer() -> Router<Pool<Sqlite>>{
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

        let sentence_db_response = sqlx::query_as::<_,CsvRowEntry>(r#"
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

        let ignored_words = sqlx::query_as::<_,WordEntry>(r#"
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
        .route("/sentence-viewer", get(|db: State<Pool<Sqlite>>, 
            template: Extension<Arc<Tera>>, 
            query: Query<SentenceViewerParams>| handler(db,template, query)))
}
