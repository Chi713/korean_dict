use korean_dict::search::Session;
use std::time::Instant;

#[tokio::main]
async fn main() {
    let now = Instant::now();
    let query = "나무".to_owned();
    let client = Session::new().await;
    let response = client.get(query).await;
    let elasped = now.elapsed();

    println!("{:?}", elasped);

    println!("{:#?}", response);
}
