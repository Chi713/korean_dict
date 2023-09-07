use clap::Parser as ClapParser;
use std::str::FromStr;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::sync::Arc;
use korean_dict_server::csv_parser;
use korean_dict_server::parser::Parser as KrParser;
use korean_dict_server::parser::ParserKind;
use korean_dict_server::search::{Entry, Session};
use axum::{
    extract,
    http::StatusCode,
    Router,
    routing::{get, post},
    Json,
    Extension,
    response::{Html, IntoResponse},
};
use serde::{Serialize, Deserialize};
use dotenvy::dotenv;
use log::info;
use tower_http::services::ServeDir;
use tera::{Tera, Context};

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


#[derive(Serialize, Deserialize, Debug)]
struct FileData {
    data: String,
}

#[tokio::main]
async fn main() {


    dotenv().ok();
    env_logger::init();
    info!("such information");

    let opt = Opt::parse();
    let sock_addr = SocketAddr::from((
        IpAddr::from_str(opt.addr.as_str()).unwrap_or(IpAddr::V6(Ipv6Addr::LOCALHOST)),
        opt.port,
    ));

    let tera = Tera::new("templates/**/*.html").unwrap();
    // let tera = Tera::default();
    // tera.add_raw_templates(vec!["index.html", include_str!("../../templates/index.html")]);

    let app = Router::new()
        .route("/", get(index))
        .route("/test", get(index))
        .route("/api/fileData", post(process_file_data))
        .nest_service("/favicon.ico", ServeDir::new("assets/favicon.ico"))
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

#[derive(Serialize, Deserialize, Debug)]
struct PostResponseData {
    sentence: String,
    sentence_entries: Vec<Entry>,
    prev_flag: bool,
    next_flag: bool,

}

async fn process_file_data(Json(body): extract::Json<FileData>) -> Result<Json<PostResponseData>, (StatusCode, String)> {

    let data = body.data;
    let mut csv_res = csv_parser::csv_parse(&data).unwrap();
    println!("csv_res {csv_res:#?}");

    let csv_data = csv_res.current_val();
    let sentence = csv_data.get("tl_subs").unwrap();

    let client = Session::new().unwrap();
    let parser = KrParser::new()
        .change_parser(ParserKind::Khaiii)
        .unwrap_or_default();

    println!("sentence: {sentence}");
    let res = parser.parse(sentence).unwrap();

    println!("parsed sentence: {:?}", res);

    let response = client.get_list(res).await.unwrap();
    println!("Searched sentence: {:#?}", response);
    Ok(Json(PostResponseData{
        sentence: sentence.to_string(),
        sentence_entries: response,
        prev_flag: false,
        next_flag: true
    }))
}

