use korean_dict::parser::{Parser, ParserKind};
use korean_dict::search::Session;
use std::time::Instant;

#[tokio::main]
async fn main() {
    let now = Instant::now();
    //let sentence = "안녕, 세상.".to_owned();
    let sentence = "제 친구 정우가 공항에서 저와 줄리아를 기다리고 있었어요.".to_owned();
    //let sentence = "사람의 말이나 행동, 성격에서 잘못된 점이나 부족한 점을 지적하여 말하다".to_owned();
    //let sentence = "재미있어요.".to_owned();
    //let sentence = "생각을 하다".to_owned();
    //let sentence = "생각하다".to_owned();
    //let query = "나무".to_owned();
    let client = Session::new().unwrap();
    let parser = Parser::new()
        .change_parser(ParserKind::Khaiii)
        .unwrap_or_default();

    println!("sentence: {sentence}");
    let res = parser.parse(sentence).unwrap();

    //let response = client.get(query).await.unwrap();

    println!("parsed sentence: {:?}", res);

    let response2 = client.get_list(res).await.unwrap();
    println!("Searched sentence: {:#?}", response2);

    let elapsed = now.elapsed();

    //println!("parsed sentence: {:?}", res);

    println!("time elasped: {:?}", elapsed);

    //println!("searched: {:#?}", response);
}
