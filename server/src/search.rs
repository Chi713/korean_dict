use futures::{stream, StreamExt};
use reqwest;
use reqwest::Client;
use roxmltree::{Document, Node};
use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;
use std::fs::File;
use dotenv::dotenv;
use anyhow::Context;
use std::io::prelude::*;

const CONCURRENT_REQUESTS: usize = 20;
const CERT_PATH: &str = "resources/certs/krdict.pem";

#[derive(Debug, Clone, PartialEq,Deserialize, Serialize)]
pub struct Entry {
    pub word: String,
    pub definition: Vec<String>,
    pub explaination: Vec<String>,
}

impl Entry {
    fn new(word: String, definition: Vec<String>, explaination: Vec<String>) -> Entry {
        Self {
            word,
            definition,
            explaination,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Session {
    client: Client,
    krdict_api_key: String,
}

impl Session {
    pub fn new() -> Result<Session, Box<dyn Error>> {
        dotenv().ok();
        println!("opening the cert file");
        let mut f = File::open(CERT_PATH).context("failed to open certificate from path")?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf).context("failed to read cert to buffer")?;
        let cert = reqwest::Certificate::from_pem(&buf)
            .context("failed to turn buffered file into reqwest certificate")?;

        let client = reqwest::Client::builder()
            .add_root_certificate(cert)
            .build()?;

        let api_key = env::var("KRDICT_API_KEY")
            .context("failed to load apikey from environment variable")?;
        Ok(Self {
            client,
            krdict_api_key: api_key,
        })
    }

    pub async fn get(&self, query: String) -> Result<Entry, Box<dyn Error + Send + Sync>> {
        let url = format!(
            "https://krdict.korean.go.kr/api/search?key={}&q={}&translated=y&trans_lang={}",
            self.krdict_api_key, query, '1'
        );

        //println!("{:?}", url);
        let response = self.client.get(&url).send().await.context("dict request failed")?;
        let data = response.text().await?;
        //println!("{:?}", data);

        //parses the data and builds Entry
        let doc: Document = roxmltree::Document::parse(&data).context("failed to parse xml")?;
        let root = doc.root().first_child().context("root doesn't exist")?;
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
                    .for_each(|child| defi.push(child.text().unwrap_or("").trim().to_owned()));
                child
                    .children()
                    .filter(|n| n.has_tag_name("trans_dfn"))
                    .for_each(|child| expl.push(child.text().unwrap_or("").trim().to_owned()));
            });

        let res = Entry::new(query.to_owned(), defi, expl);
        Ok(res)
    }

    pub async fn get_list(&self, words: Vec<String>) -> Result<Vec<Entry>, Box<dyn Error + Send + Sync>> {
        stream::iter(words.into_iter().map(|word| self.get(word)))
            .buffered(CONCURRENT_REQUESTS)
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect()
        //bodies

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
        let query = "나무";
        let client = Session::new().unwrap();
        let response = client.get(query.to_owned()).await.unwrap();
        assert_eq!(
            response,
            Entry {
                word: "나무".into(),
                definition: vec!["tree".into(), "wood".into(), "timber; log".into()],
                explaination: vec![
                    "A plant with a hard stem, branches and leaves.".into(),
                    "The material used to build a house or to make furniture.".into(),
                    "The trunk or branches of a tree cut to be used as firewood.".into()
                ],
            }
        );
    }

    #[tokio::test]
    async fn test_session_get_list() {
        let query = vec!["공항".to_owned(), "기다리다".to_owned()];
        let client = Session::new().unwrap();
        let response = client.get_list(query).await.unwrap();
        assert_eq!(
            response,
            [Entry {
                word: "공항".into(),
                definition: vec!["airport".into()],
                explaination: vec!["A place for airplanes to land and take off.".into()],
            }, Entry {
                    word: "기다리다".to_owned(),
                    definition: vec!["wait".to_owned()],
                    explaination: vec!["To spend time until a person or time comes or a certain event is realized.".to_owned()],
            }]);
    }
}
