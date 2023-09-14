use axum::extract::State;
use clap::Parser as ClapParser;
use sqlx::QueryBuilder;
use std::str::FromStr;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::sync::Arc;
use std::env;
// use korean_dict_server::csv_parser;
use korean_dict::parser::Parser as KrParser;
use korean_dict::parser::{LanguageParser, KhaiiiParser};
use korean_dict::search::Session;
use korean_dict::srt_parser::srt_parser;
use axum::{
    extract::{Multipart, Query},
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

#[derive(Clone, FromRow, Debug)]
struct Vocab {
    id: i64,
    word: String
}

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
        .route("/", get(index))
        .route("/api/fileData", post(process_file_data))
        .route("/sentence-viewer", get(sentence_viewer))
        .with_state(db)
        .nest_service("/favicon.ico", ServeDir::new("public/favicon.ico"))
        .layer(Extension(Arc::new(tera)));

    info!("listening on http://{}", sock_addr);

    axum::Server::bind(&sock_addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn index(
    Extension(templates): Extension<Arc<Tera>>,
) -> impl IntoResponse {
    let context = Context::new();
    Html(templates.render("index.html", &context).unwrap())
}


#[derive(Clone, FromRow, Debug)]
struct StrEntry {
    srt_id:i64,
    file_name: String
}

#[derive(Clone, FromRow, Debug)]
struct SubtitleEntry {
    subtitle_id: i64,
    srt_id: i64,
    subtitle_order: i64,
    subtitle_start: String,
    subtitle_end: String,
    subtitle_text: String,
}

async fn process_file_data(State(db): State<SqlitePool> ,mut multipart: Multipart) -> Result<Redirect, (StatusCode, String)> {
    let mut srt_id = 0;
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

            // srt parsing
            let srt_res = srt_parser(data).unwrap(); 

            srt_id = sqlx::query(r#"INSERT INTO srt (file_name) VALUES (?);"#)
                .bind(file_name)
                .execute(&db)
                .await
                .unwrap()
                .last_insert_rowid();

            let mut subtitle_query_builder: QueryBuilder<Sqlite> = QueryBuilder::new(
                "INSERT INTO subtitles (
                    srt_id, subtitle_order, subtitle_start, subtitle_end, subtitle_text
                ) " 
            ); 

            subtitle_query_builder.push_values(srt_res.subtitles.iter().take(BIND_LIMIT/5), |mut b, sub| {
                b.push_bind(srt_id)
                    .push_bind(sub.order)
                    .push_bind(sub.time_start.render())
                    .push_bind(sub.time_end.render())
                    .push_bind(sub.text.clone());
            });

            let subtitle_query = subtitle_query_builder.build();
            subtitle_query.execute(&db).await.unwrap();

            let srt_db_data = sqlx::query_as::<_,SubtitleEntry>(r#"
                SELECT subtitle_id, srt_id, subtitle_order, subtitle_start, subtitle_end, subtitle_text 
                FROM subtitles WHERE srt_id = ? ;"#)
                .bind(srt_id)
                .fetch_all(&db)
                .await
                .unwrap();
                
            println!("{:?}", srt_db_data);
        }
    }

    let sentence_viewer_uri = format!("/sentence-viewer?srt_id={}", srt_id);
    Ok(Redirect::to(&sentence_viewer_uri))
}

#[derive(Debug, Deserialize)]
struct SentenceViewerParams {
    srt_id: u32,
    sentence: Option<u32>
}

async fn sentence_viewer(
    State(db): State<SqlitePool>,
    Extension(templates): Extension<Arc<Tera>>,
    Query(params): Query<SentenceViewerParams> 
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let srt_id = params.srt_id;
    println!("{:?}", srt_id);

    let sentence_db_response = sqlx::query_as::<_,SubtitleEntry>(r#"
            SELECT subtitle_id, srt_id, subtitle_order, subtitle_start, subtitle_end, subtitle_text 
            FROM subtitles WHERE srt_id = ?;"#)
        .bind(srt_id)
        .fetch_one(&db)
        .await
        .unwrap();

    let sentence = &sentence_db_response.subtitle_text;

    let client = Session::new().unwrap();
    let parser = KrParser::new(KhaiiiParser::new());

    println!("sentence: {sentence}");
    let parsed_sentence = parser.parser.parse(sentence).unwrap();
    println!("parsed sentence: {:?}", parsed_sentence);
    let searched_words_list = client.get_list(parsed_sentence).await.unwrap();
    println!("Searched words list: {:#?}", searched_words_list);

    let mut context = Context::new();
    context.insert("srt_id", &srt_id);
    context.insert("words_list", &searched_words_list);
    Ok(Html(templates.render("sentence_viewer.html", &context).unwrap()))
}
