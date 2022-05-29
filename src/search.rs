use reqwest;
use reqwest::Client;
use roxmltree::{Document, Node};
use std::fs::File;
use std::io::prelude::*;
//use std::time::Instant;

#[derive(Debug)]
pub struct Entry {
    pub word: String,
    pub definition: Vec<String>,
    pub explaination: Vec<String>,
}

impl Entry {
    pub fn new(word: String, definition: Vec<String>, explaination: Vec<String>) -> Entry {
        Entry {
            word,
            definition,
            explaination,
        }
    }
}

pub struct Session {
    client: Client,
    api_key: String,
}

impl Session {
    pub async fn new() -> Session {
        let mut f = File::open("certs/krdict.pem").expect("Certificate file is missing");
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)
            .expect("error reading Certificate file");
        let cert = reqwest::Certificate::from_pem(&buf).expect("error creating Certificate");

        let client = reqwest::Client::builder()
            .add_root_certificate(cert)
            .build()
            .expect("error initilizing client");

        let api_key = Session::api_key();
        Session { client, api_key }
    }

    fn api_key() -> String {
        let mut f = File::open(".apikey").expect("failed to open file");
        let mut api_key = String::new();
        f.read_to_string(&mut api_key)
            .expect("error fetching apikey from file");
        api_key
    }

    pub async fn get(&self, query: String) -> Entry {
        let url = format!(
            "https://krdict.korean.go.kr/api/search?key={}&q={}&translated={}&trans_lang={}",
            self.api_key, query, 'y', '1'
        );
        println!("{}", url);

        let response = self.client.get(&url).send().await;
        let data = response.unwrap().text().await.expect("reqwest error");

        let data = data.replace("\n", "").replace("\t", "");
        Session::parse(data, query)
    }

    fn parse(data: String, query: String) -> Entry {
        let doc: Document = roxmltree::Document::parse(&data).unwrap();
        let root = doc.root().first_child().unwrap();
        let mut defi = Vec::new();
        let mut expl = Vec::new();

        let result = root
            .children()
            .filter(|n| n.has_tag_name("item") & Session::has_child_tag(n, "word", &query))
            .flat_map(move |s| s.children())
            .filter(|n| n.has_tag_name("sense"))
            .flat_map(move |s| s.children())
            .filter(|n| n.has_tag_name("translation"));

        for child in result {
            for child in child.children().filter(|n| n.has_tag_name("trans_word")) {
                defi.push(child.text().unwrap_or_default().to_owned());
            }
            for child in child.children().filter(|n| n.has_tag_name("trans_dfn")) {
                expl.push(child.text().unwrap_or_default().to_owned());
            }
        }

        Entry::new(query.to_string(), defi, expl)
    }

    fn has_child_tag(node: &Node, tag: &str, query: &str) -> bool {
        let mut flag = false;

        for child in node.children().filter(|n| n.has_tag_name(tag)) {
            flag = Some(query) == child.text();
        }
        flag
    }
}
