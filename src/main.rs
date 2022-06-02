use korean_dict::parser::Parser;
use korean_dict::search::Session;
use std::time::Instant;

#[tokio::main]
async fn main() {
    let now = Instant::now();
    let sentence = "안녕, 새상.".to_owned();
    let query = "나무".to_owned();
    let client = Session::new().await;
    let parse = Parser::new();

    let res = parse.parse(sentence).unwrap();
    println!("parsed sentence: {:?}", res);

    let response = client.get(query).await.unwrap();
    let elasped = now.elapsed();

    println!("time elasped: {:?}", elasped);

    println!("searched: {:#?}", response);
}
