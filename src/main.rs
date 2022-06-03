use korean_dict::parser::{Parser, ParserKind, Sentence};
use korean_dict::search::Session;
use std::time::Instant;

#[tokio::main]
async fn main() {
    let now = Instant::now();
    let test_sentence = "안녕, 새상.".to_owned();
    let query = "나무".to_owned();
    let client = Session::new().await;
    let sentence = Sentence::new(test_sentence.clone());
    let parser = Parser::new();
    let parser = parser.change_parser(ParserKind::Khaiii);

    let res = sentence.parse(&parser).unwrap();
    println!("parsed sentence: {:?}", res);

    let response = client.get(query).await.unwrap();
    let elasped = now.elapsed();

    println!("time elasped: {:?}", elasped);

    println!("searched: {:#?}", response);
}
