use clap::Parser as ClapParser;
use std::str::FromStr;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::sync::Arc;
use std::env;
use korean_dict::routes;
use axum::{
    Router,
    Extension,
};
use dotenvy::dotenv;
use log::info;
use tower_http::services::ServeDir;
use tera::Tera;
use sqlx::{migrate::MigrateDatabase ,Sqlite, SqlitePool};

// Setup the command line interface with clap.
#[derive(ClapParser, Debug)]
#[clap(name = "server", about = "A server for our wasm project!")]
struct Opt {
    /// set the log level
    #[clap(short = 'l', long = "log", default_value = "debug")]
    log_level: String,
    /// set the listen addr
    #[clap(short = 'a', long = "addr", default_value = "0.0.0.0")]
    addr: String,

    /// set the listen port
    #[clap(short = 'p', long = "port", default_value = "8080")]
    port: u16,

    /// set the directory where static files are to be found
    #[clap(long = "static-dir", default_value = "./dist")]
    static_dir: String,
}

const DB_URL: &str = "sqlite://database.db";

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
        .merge(routes::csv_processing::process_file_data())
        .merge(routes::views::view())
        .merge(routes::views::sentence_viewer())
        .merge(routes::views::flashcard_entry_post())
        .merge(routes::views::flashcard_entry_patch())
        .merge(routes::views::flashcard_entry_delete())
        .with_state(db)
        .merge(routes::index::index())
        .nest_service("/favicon.ico", ServeDir::new("public/favicon.ico"))
        .layer(Extension(Arc::new(tera)));

    info!("listening on http://{}", sock_addr);

    axum::Server::bind(&sock_addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
