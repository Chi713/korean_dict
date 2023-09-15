use std::error::Error;
use std::collections::HashSet;
use std::process::Command;
use std::sync::Arc;
use serde_json;

const EXCEPTIONS: &[&str] = &[
    "JKS", "JKC", "JKG", "JKO", "JKB", "JKV", "JKQ", "JX", "JC", "SP", "SF", "SE", "SS", "EC",
    "EP", "EF", "ETM",
];

pub trait LanguageParser {
    fn parse(&self, sentence: &str) -> Result<Vec<String>, Box<dyn Error>>;
}

pub struct Parser<T> {
     pub parser: Arc<T>
}


impl<T> Parser<T> 
where T: LanguageParser
{
    pub fn new(parser: T) -> Self {
        Self { parser: Arc::new(parser) }
    }
}

pub struct KomoranParser {
}

impl LanguageParser for KomoranParser {
    // can only be used strictly synchronously
    fn parse(&self, sentence: &str) -> Result<Vec<String>, Box<dyn Error>> {
        let result_output = Command::new("python3")
            .arg("py/komoran_parser.py")
            .arg(sentence)
            .output()
            .expect("Command failed, sentence not parsed");
        let result_json = String::from_utf8(result_output.stdout).unwrap();
        let result_json = result_json.trim();
        println!("result_json: {:?}",result_json);
        let result: Vec<(String, String)> = serde_json::from_str(result_json).unwrap();

        let mut ex_tags: Vec<&str> = vec!["XPN", "NP", "VX"];
        ex_tags.extend(EXCEPTIONS.iter().copied());

        let filtered_result: Vec<(String, String)> = result
            .into_iter()
            .filter(|x| !has_ban_morph(x))
            .filter(|x| !ex_tags.contains(&x.1.as_str()))
            .collect();

        let words_list: Vec<String> = remove_dup(komoran_word_processor(filtered_result));

        Ok(words_list)
    }
}

fn komoran_word_processor(data: Vec<(String, String)>) -> Vec<String> {
    let verb_tags = vec!["VV", "XSV", "XSA", "VA", "V"];
    let stem_tags = vec!["XSV", "XSA"];

    let mut output_data: Vec<String> = Vec::new();
    for (mut word, tag) in data {
        if stem_tags.contains(&tag.as_ref()) {
            let mut temp_word: String = output_data.pop().unwrap();
            temp_word.push_str(&word);
            word = temp_word;
        }

        if verb_tags.contains(&tag.as_ref()) {
            word.push('다');
        }

        output_data.push(word.to_owned());
        }
    output_data
}

pub struct KhaiiiParser {
}

impl KhaiiiParser {
    pub fn new() -> KhaiiiParser {
        KhaiiiParser{}
    }
}

impl LanguageParser for KhaiiiParser {
    fn parse(&self, sentence: &str) -> Result<Vec<String>,Box<dyn Error>> {
        let result_output = Command::new("python3")
            .arg("py/khaiii_parser.py")
            .arg(sentence)
            .output()
            .expect("Command failed, sentence not parsed");

        let result_json = String::from_utf8(result_output.stdout).unwrap();
        let result_json = result_json.trim();
        println!("result_json: {:?}",result_json);
        let result: Vec<Vec<(String, String)>> = serde_json::from_str(result_json).unwrap();

        println!("res tag: {:?}", result);

        let mut ex_tags = vec!["NNP", "NP", "VX"];
        ex_tags.extend(EXCEPTIONS.iter().copied());

        let mut words_list = result.into_iter().flat_map(|word| {
            let word_filtered: Vec<(String, String)> = word.into_iter()
                .filter(|m| !ex_tags.contains(&m.1.as_str()))
                .filter(|m| !has_ban_morph(m))
                .collect();
            khaiii_word_processor(word_filtered)
        })
        .collect();

        words_list= remove_dup(words_list);

        Ok(words_list)
    }
}

fn khaiii_word_processor(data: Vec<(String, String)>) -> Vec<String> {
    let verb_tags = vec!["VV", "VA", "XSV", "XSA"];
    let stem_tags = vec!["XSV", "XSA"];

    let mut output_data: Vec<String> = Vec::new();
    
    for (mut word, tag) in data {
        if stem_tags.contains(&tag.as_ref()) {
            let mut temp_word: String = output_data.pop().unwrap();
            temp_word.push_str(&word);
            word = temp_word;
        }

        if verb_tags.contains(&tag.as_ref()) {
            word.push('다');
        }
        output_data.push(word.to_owned());
    }
    output_data
}

fn remove_dup (mut list: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    list.retain(|s| seen.insert(s.clone()));
    list

}

fn has_ban_morph(word: &(String, String)) -> bool {
    let banned_verb_tags = vec![
        ("하".to_owned(), "VV".to_owned()),
        ("되".to_owned(), "VV".to_owned()),
    ];
    banned_verb_tags.iter().any(|e| e == word)
}

#[cfg(test)]
mod tests {
    // use super::*;
    // 
    // #[test]
    // fn test_parser_new() {
    //     let parser = Parser::new();
    //     assert_eq!(
    //         parser,
    //         Parser {
    //             parser: ParserKind::Komoran,
    //         }
    //     );
    // }
    //
    // #[test]
    // fn test_default_parser() {
    //     let parser = Parser::new();
    //     assert_eq!(parser.parser_type(), ParserKind::Komoran);
    // }
    //
    // #[test]
    // fn test_change_parser() {
    //     let parser = Parser::new().change_parser(ParserKind::Khaiii).unwrap();
    //     assert_eq!(parser.parser_type(), ParserKind::Khaiii);
    // }
    //
    // #[test]
    // fn test_komoran_parser() {
    //     let parser = Parser::new().change_parser(ParserKind::Komoran).unwrap();
    //     let test_sentence = "안녕, 세상.";
    //     let res = parser.parse(test_sentence).unwrap();
    //     assert_eq!(res, ["안녕".to_owned(), "세상".to_owned(),]);
    // }
    //
    // #[test]
    // fn test_komoran_parser2() {
    //     let parser = Parser::new();
    //     let test_sentence = "제 친구 정우가 공항에서 저와 줄리아를 기다리고 있었어요.";
    //     let res = parser.parse(test_sentence).unwrap();
    //     assert_eq!(
    //         res,
    //         [
    //             "친구".to_owned(),
    //             "정우".to_owned(),
    //             "공항".to_owned(),
    //             "줄리아".to_owned(),
    //             "기다리다".to_owned()
    //         ]
    //     );
    // }
    //
    // #[test]
    // fn test_komoran_parser3() {
    //     let parser = Parser::new();
    //     let test_sentence = "생각하다";
    //     let res = parser.parse(test_sentence).unwrap();
    //     assert_eq!(res, ["생각하다".to_owned()]);
    // }
    //
    // #[test]
    // fn test_komoran_parser4() {
    //     let parser = Parser::new();
    //     let test_sentence = "생각을 하다";
    //     let res = parser.parse(test_sentence).unwrap();
    //     assert_eq!(res, ["생각".to_owned()]);
    // }
    //
    // #[test]
    // fn test_komoran_parser5() {
    //     let parser = Parser::new();
    //     let test_sentence = "다시 또 다시";
    //     let res = parser.parse(test_sentence).unwrap();
    //     assert_eq!(res, ["다시".to_owned(), "또".to_owned()]);
    // }
    //
    // #[test]
    // fn test_khaiii_parser() {
    //     // if Parser::has_khaiii().unwrap() {
    //         let parser = Parser::new().change_parser(ParserKind::Khaiii).unwrap();
    //         let test_sentence = "안녕, 새상.";
    //         let res = parser.parse(test_sentence).unwrap();
    //         assert_eq!(res, ["안녕".to_owned(), "새상".to_owned()]);
    //     // }
    // }
    //
    // //test 2-4 check if all of the appropriate particles are discarded and proper stems are applied to verbs
    // #[test]
    // fn test_khaiii_parser2() {
    //     // if Parser::has_khaiii().unwrap() {
    //         let parser = Parser::new().change_parser(ParserKind::Khaiii).unwrap();
    //         let test_sentence =
    //             "제 친구 정우가 공항에서 저와 줄리아를 기다리고 있었어요.";
    //         let res = parser.parse(test_sentence).unwrap();
    //         assert_eq!(
    //             res,
    //             ["친구".to_owned(), "공항".to_owned(), "기다리다".to_owned()]
    //         );
    //     // }
    // }
    //
    // #[test]
    // fn test_khaiii_parser3() {
    //     // if Parser::has_khaiii().unwrap() {
    //         let parser = Parser::new().change_parser(ParserKind::Khaiii).unwrap();
    //         let test_sentence = "생각하다";
    //         let res = parser.parse(test_sentence).unwrap();
    //         assert_eq!(res, ["생각하다".to_owned()]);
    //     // }
    // }
    //
    // #[test]
    // fn test_khaiii_parser4() {
    //     // if Parser::has_khaiii().unwrap() {
    //         let parser = Parser::new().change_parser(ParserKind::Khaiii).unwrap();
    //         let test_sentence = "생각을 하다";
    //         let res = parser.parse(test_sentence).unwrap();
    //         assert_eq!(res, ["생각".to_owned()]);
    //     // }
    // }
    //
    // #[test]
    // fn test_khaiii_parser5() {
    //     // if Parser::has_khaiii().unwrap() {
    //         let parser = Parser::new().change_parser(ParserKind::Khaiii).unwrap();
    //         let test_sentence = "다시 시작해, 다시";
    //         let res = parser.parse(test_sentence).unwrap();
    //         assert_eq!(res, ["다시".to_owned(), "시작하다".to_owned()]);
    //     // }
    // }
}
