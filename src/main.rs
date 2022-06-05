use korean_dict::parser::{Parser, ParserKind};
use korean_dict::search::Session;
use std::time::Instant;

#[tokio::main]
async fn main() {
    //let now = Instant::now();
    let sentence = "안녕, 새상.".to_owned();
    let query = "나무".to_owned();
    let now = Instant::now();
    let client = Session::new().unwrap();
    let elapsed = now.elapsed();
    //let sentence = Sentence::new(test_sentence);
    let parser = Parser::new()
        .change_parser(ParserKind::Khaiii)
        .unwrap_or_default();

    let res = parser.parse(sentence).unwrap();

    let response = client.get(query).await.unwrap();
    //let elapsed = now.elapsed();

    println!("parsed sentence: {:?}", res);

    println!("time elasped: {:?}", elapsed);

    println!("searched: {:#?}", response);
}
