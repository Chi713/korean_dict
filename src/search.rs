use futures::{stream, StreamExt};
use reqwest;
use reqwest::Client;
use roxmltree::{Document, Node};
use std::env;
use std::error::Error;
use std::fs::File;
use std::io;
use std::io::prelude::*;

const CONCURRENT_REQUESTS: usize = 20;

#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub word: String,
    pub definition: Vec<String>,
    pub explaination: Vec<String>,
}

impl Entry {
    fn new(word: String, definition: Vec<String>, explaination: Vec<String>) -> Entry {
        Entry {
            word,
            definition,
            explaination,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Session {
    client: Client,
    api_key: String,
}

impl Session {
    pub fn new() -> Result<Session, Box<dyn Error>> {
        let mut f = File::open("resources/certs/krdict.pem")?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;
        let cert = reqwest::Certificate::from_pem(&buf)?;

        let client = reqwest::Client::builder()
            .add_root_certificate(cert)
            .build()?;

        let api_key = Session::api_key()?;
        Ok(Session { client, api_key })
    }

    fn api_key() -> Result<String, io::Error> {
        let args: Vec<String> = env::args().collect();
        println!("args: {:?}", args);
        println!("env args: {:?}", env::var("KRDICT_API_KEY").is_ok());
        let api_key: String;
        if args.len() > 1 {
            println!("used cmd passed env key");
            api_key = args[1].clone();
        } else if env::var("KRDICT_API_KEY").is_ok() {
            println!("used env set key");
            api_key = env::var("KRDICT_API_KEY").unwrap().into();
        } else {
            println!("used file");
            let mut f = File::open("./.apikey")?;
            let mut buf = String::new();
            f.read_to_string(&mut buf)?;
            api_key = buf.trim().into();
            println!("apikey: {}", api_key);
        }
        Ok(api_key)
    }



    pub async fn get(&self, query: String) -> Result<Entry, Box<dyn Error>> {
        let url = format!(
            "https://krdict.korean.go.kr/api/search?key={}&q={}&translated=y&trans_lang={}",
            self.api_key, query, '1'
        );

        println!("{:?}", url);
        let response = self.client.get(&url).send().await?;
        let data = response.text().await?;
        //println!("{:?}", data);
        let res = Session::parse(data, query)?;
        Ok(res)
    }

    fn parse(data: String, query: String) -> Result<Entry, roxmltree::Error> {
        let doc: Document = roxmltree::Document::parse(&data)?;
        let root = doc.root().first_child().unwrap();
        let mut defi = Vec::new();
        let mut expl = Vec::new();

        root.children()
            .filter(|n| n.has_tag_name("item") & Session::has_child_tag(n, "word", &query))
            .flat_map(|s| s.children())
            .filter(|n| n.has_tag_name("sense"))
            .flat_map(|s| s.children())
            .filter(|n| n.has_tag_name("translation"))
            .for_each(|child| {
                child
                    .children()
                    .filter(|n| n.has_tag_name("trans_word"))
                    .for_each(|child| defi.push(child.text().unwrap_or("").to_owned()));
                child
                    .children()
                    .filter(|n| n.has_tag_name("trans_dfn"))
                    .for_each(|child| expl.push(child.text().unwrap_or("").to_owned()));
            });

        let res = Entry::new(query.to_owned(), defi, expl);
        Ok(res)
    }

    pub async fn get_list(&self, words: Vec<String>) -> Result<Vec<Entry>, Box<dyn Error>> {
        let bodies = stream::iter(words.into_iter().map(|word| self.get(word)))
            .buffered(CONCURRENT_REQUESTS)
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect();
        bodies

        //TODO add caching
    }

    fn has_child_tag(node: &Node, tag: &str, query: &str) -> bool {
        let mut flag = false;

        node.children()
            .filter(|n| n.has_tag_name(tag))
            .for_each(|child| flag = Some(query) == child.text());

        flag
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_new() {
        Session::new().unwrap();
    }

    #[tokio::test]
    async fn test_session_get() {
        let query = "나무".to_owned();
        let client = Session::new().unwrap();
        let response = client.get(query).await.unwrap();
        assert_eq!(
            response,
            Entry {
                word: "나무".into(),
                definition: vec!("tree".into(), "wood".into(), "timber; log".into()),
                explaination: vec!(
                    "A plant with a hard stem, branches and leaves.".into(),
                    "The material used to build a house or to make furniture.".into(),
                    "The trunk or branches of a tree cut to be used as firewood.".into()
                ),
            }
        );
    }
}
