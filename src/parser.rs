use pyo3::prelude::*;
use std::error::Error;

const EXCEPTIONS: &[&str] = &[
    "JKS", "JKC", "JKG", "JKO", "JKB", "JKV", "JKQ", "JX", "JC", "SP", "SF", "SE", "SS", "EC",
    "EP", "EF", "ETM",
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParserKind {
    Khaiii,
    Komoran,
}

impl Default for ParserKind {
    fn default() -> Self {
        ParserKind::Komoran
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Parser {
    parser: ParserKind,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            parser: ParserKind::default(),
        }
    }

    pub fn parse(&self, sentence: String) -> PyResult<Vec<String>> {
        match self.parser_type() {
            ParserKind::Khaiii => khaiii_parse(sentence),
            ParserKind::Komoran => komoran_parse(sentence),
        }
    }

    //slient error being handled here MAKE SURE TO FIX
    pub fn change_parser(mut self, parser: ParserKind) -> Result<Self, Box<dyn Error>> {
        let flag = Self::has_khaiii()?;
        let parser = match (parser, flag) {
            (ParserKind::Khaiii, true) => ParserKind::Khaiii,
            (ParserKind::Khaiii, false) => ParserKind::Komoran,
            (ParserKind::Komoran, _) => ParserKind::Komoran,
        };
        self.parser = parser;
        Ok(self)
    }

    pub fn has_khaiii() -> PyResult<bool> {
        let res: PyResult<bool> = Python::with_gil(|py| {
            let check = PyModule::from_code(
                py,
                "def check(*args):
                try:
                    import khaiii
                    return True
                except ImportError:
                    return False",
                "",
                "",
            )?;
            let result = check.getattr("check")?.call0()?.extract()?;
            Ok(result)
        });
        res
    }

    pub fn parser_type(&self) -> ParserKind {
        println!("{:?}", self.parser);
        self.parser
    }
}

//can only be used strictly synchronously
fn komoran_parse(sentence: String) -> PyResult<Vec<String>> {
    let res: PyResult<Vec<(String, String)>> = Python::with_gil(|py| {
        let fun = PyModule::from_code(
            py,
            "def parse(*arg):
                from konlpy.tag import Komoran
                return Komoran().pos(arg[0])",
            "",
            "",
        )?;

        let result = fun.getattr("parse")?.call1((sentence,))?.extract();
        result
    });
    let mut words_list: Vec<String> = Vec::new();
    let verb_tags = vec!["VV", "XSV", "XSA", "VA", "V"];
    let stem_tags = vec!["XSV", "XSA"];
    let mut ex_tags: Vec<&str> = vec!["XPN", "NP", "VX"];
    ex_tags.extend(EXCEPTIONS.iter().copied());

    let res = res?;
    //println!("res tag: {:?}", res);

    res.into_iter()
        .filter(|m| !ex_tags.contains(&m.1.as_str()) & !has_ban_morph(m))
        .for_each(|morphs| {
            let (mut word, tag) = morphs;
            if stem_tags.contains(&tag.as_ref()) {
                let mut temp_word: String = words_list.pop().unwrap();
                temp_word.push_str(&word);
                word = temp_word;
            }

            if verb_tags.contains(&tag.as_ref()) {
                word.push('다');
            }

            words_list.push(word.to_owned());
        });

    Ok(words_list)
}

fn khaiii_parse(sentence: String) -> PyResult<Vec<String>> {
    let res: PyResult<Vec<Vec<(String, String)>>> = Python::with_gil(|py| {
        let fun = PyModule::from_code(
            py,
            "def parse(*arg):
                from khaiii import KhaiiiApi

                words_list = list()
                for word in [w.morphs for w in KhaiiiApi().analyze(arg[0])]:
                    words_list.append([(mor.lex, mor.tag) for mor in word])
                return words_list",
            "",
            "",
        );

        let result = fun?.getattr("parse")?.call1((sentence,))?.extract();

        result
    });

    let mut words_list: Vec<String> = Vec::new();
    let verb_tags = vec!["VV", "VA", "XSV", "XSA"];
    let stem_tags = vec!["XSV", "XSA"];
    let mut ex_tags = vec!["NNP", "NP", "VX"];
    //let exception_words = vec!["하다", "되다", "있다", "없다", "나다"];
    ex_tags.extend(EXCEPTIONS.iter().copied());

    let res = res?;
    println!("res tag: {:?}", res);

    res.into_iter().for_each(|w| {
        w.into_iter()
            .filter(|m| !ex_tags.contains(&m.1.as_str()) & !has_ban_morph(m))
            .for_each(|morphs| {
                let (mut word, tag) = morphs;
                if stem_tags.contains(&tag.as_ref()) {
                    let mut temp_word: String = words_list.pop().unwrap();
                    temp_word.push_str(&word);
                    word = temp_word;
                }

                if verb_tags.contains(&tag.as_ref()) {
                    word.push('다');
                }
                //if !exception_words.contains(&word) {
                words_list.push(word.to_owned());
                //}
            });
    });

    Ok(words_list)
}

fn has_ban_morph(word: &(String, String)) -> bool {
    let mut flag: bool = false;
    let banned_verb_tags = vec![
        ("하".to_owned(), "VV".to_owned()),
        ("되".to_owned(), "VV".to_owned()),
    ];
    if banned_verb_tags.iter().any(|e| e == word) {
        flag = true;
    }
    flag
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    fn test_parser_new() {
        let parser = Parser::new();
        assert_eq!(
            parser,
            Parser {
                parser: ParserKind::Komoran
            }
        );
    }

    #[test]
    fn test_default_parser() {
        let parser = Parser::new();
        assert_eq!(parser.parser_type(), ParserKind::Komoran);
    }

    #[test]
    fn test_change_parser() {
        let parser = Parser::new().change_parser(ParserKind::Khaiii).unwrap();
        if Parser::has_khaiii().unwrap() {
            assert_eq!(parser.parser_type(), ParserKind::Khaiii);
        } else {
            assert_eq!(parser.parser_type(), ParserKind::Komoran);
        }
    }

    #[test]
    #[serial]
    fn test_komoran_parser() {
        let parser = Parser::new().change_parser(ParserKind::Komoran).unwrap();
        let test_sentence = "안녕, 세상.".to_owned();
        let res = parser.parse(test_sentence).unwrap();
        assert_eq!(res, ["안녕".to_owned(), "세상".to_owned(),]);
    }

    #[test]
    #[serial]
    fn test_komoran_parser2() {
        let parser = Parser::new();
        let test_sentence = "제 친구 정우가 공항에서 저와 줄리아를 기다리고 있었어요.".to_owned();
        let res = parser.parse(test_sentence).unwrap();
        assert_eq!(
            res,
            [
                "친구".to_owned(),
                "정우".to_owned(),
                "공항".to_owned(),
                "줄리아".to_owned(),
                "기다리다".to_owned()
            ]
        );
    }

    #[test]
    #[serial]
    fn test_komoran_parser3() {
        let parser = Parser::new();
        let test_sentence = "생각하다".to_owned();
        let res = parser.parse(test_sentence).unwrap();
        assert_eq!(res, ["생각하다".to_owned()]);
    }

    #[test]
    #[serial]
    fn test_komoran_parser4() {
        let parser = Parser::new();
        let test_sentence = "생각을 하다".to_owned();
        let res = parser.parse(test_sentence).unwrap();
        assert_eq!(res, ["생각".to_owned()]);
    }

    #[test]
    #[serial]
    fn test_khaiii_parser() {
        if Parser::has_khaiii().unwrap() {
            let parser = Parser::new().change_parser(ParserKind::Khaiii).unwrap();
            let test_sentence = "안녕, 새상.".to_owned();
            let res = parser.parse(test_sentence).unwrap();
            assert_eq!(res, ["안녕".to_owned(), "새상".to_owned()]);
        }
    }

    //test 2-4 check if all of the appropriate particles are discarded and proper stems are applied to verbs
    #[test]
    fn test_khaiii_parser2() {
        if Parser::has_khaiii().unwrap() {
            let parser = Parser::new().change_parser(ParserKind::Khaiii).unwrap();
            let test_sentence =
                "제 친구 정우가 공항에서 저와 줄리아를 기다리고 있었어요.".to_owned();
            let res = parser.parse(test_sentence).unwrap();
            assert_eq!(
                res,
                ["친구".to_owned(), "공항".to_owned(), "기다리다".to_owned()]
            );
        }
    }

    #[test]
    fn test_khaiii_parser3() {
        if Parser::has_khaiii().unwrap() {
            let parser = Parser::new().change_parser(ParserKind::Khaiii).unwrap();
            let test_sentence = "생각하다".to_owned();
            let res = parser.parse(test_sentence).unwrap();
            assert_eq!(res, ["생각하다".to_owned()]);
        }
    }

    #[test]
    fn test_khaiii_parser4() {
        if Parser::has_khaiii().unwrap() {
            let parser = Parser::new().change_parser(ParserKind::Khaiii).unwrap();
            let test_sentence = "생각을 하다".to_owned();
            let res = parser.parse(test_sentence).unwrap();
            assert_eq!(res, ["생각".to_owned()]);
        }
    }
}
