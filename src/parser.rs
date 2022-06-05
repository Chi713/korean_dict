use pyo3::prelude::*;
use std::error::Error;

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
        let flag = Self::check_khaiii_install()?;
        let parser = match (parser, flag) {
            (ParserKind::Khaiii, true) => ParserKind::Khaiii,
            (ParserKind::Khaiii, false) => ParserKind::Komoran,
            (ParserKind::Komoran, _) => ParserKind::Komoran,
        };
        self.parser = parser;
        Ok(self)
    }

    pub fn check_khaiii_install() -> PyResult<bool> {
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

        let result = fun.getattr("parse")?.call1((sentence,))?.extract()?;
        Ok(result)
    });
    let mut words: Vec<String> = Vec::new();
    let mut word: &str;
    let mut tag: &str;
    let exceptions = vec![
        "JKS", "JKC", "JKG", "JKO", "JKB", "JKV", "JKQ", "JX", "JC", "SP", "SF",
    ];

    for parts in res? {
        (word, tag) = (parts.0.as_str(), parts.1.as_str());
        if !exceptions.contains(&tag) {
            words.push(word.to_owned());
        }
    }

    Ok(words)
}

fn khaiii_parse(sentence: String) -> PyResult<Vec<String>> {
    let res: PyResult<Vec<Vec<(String, String)>>> = Python::with_gil(|py| {
        let fun = PyModule::from_code(
            py,
            "def parse(*arg):
                from khaiii import KhaiiiApi

                words = list()
                for word in [w.morphs for w in KhaiiiApi().analyze(arg[0])]:
                    words.append([(mor.lex, mor.tag) for mor in word])
                return words",
            "",
            "",
        );

        let result = fun?.getattr("parse")?.call1((sentence,))?.extract()?;

        Ok(result)
    });
    //temporary fix to get code to run. TODO need to do actual parsing
    //let mut words: Vec<(String, String)> = Vec::new();

    //res?.to_owned_iter().for_each(|sent| {
    //    println!("{:?}", sent);
    //    words.push(sent[0].clone());
    //});

    let mut words: Vec<String> = Vec::new();
    let mut word: &str;
    let mut tag: &str;
    let exceptions = vec![
        "JKS", "JKC", "JKG", "JKO", "JKB", "JKV", "JKQ", "JX", "JC", "SP", "SF",
    ];

    for parts in res? {
        for thing in parts {
            (word, tag) = (thing.0.as_str(), thing.1.as_str());
            if !exceptions.contains(&tag) {
                words.push(word.to_owned());
            }
        }
    }

    Ok(words)
}

#[cfg(test)]
mod tests {
    use super::*;

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
        if Parser::check_khaiii_install().unwrap() {
            assert_eq!(parser.parser_type(), ParserKind::Khaiii);
        } else {
            assert_eq!(parser.parser_type(), ParserKind::Komoran);
        }
    }

    #[test]
    //#[ignore]
    fn test_khaiii_parser() {
        if Parser::check_khaiii_install().unwrap() {
            let parser = Parser::new().change_parser(ParserKind::Khaiii).unwrap();
            let test_sentence = "안녕, 새상.".to_owned();
            let res = parser.parse(test_sentence).unwrap();
            assert_eq!(res, ["안녕".to_owned(), "새상".to_owned()]);
        }
    }

    #[test]
    fn test_komoran_parser() {
        let parser = Parser::new().change_parser(ParserKind::Komoran).unwrap();
        let test_sentence = "안녕, 새상.".to_owned();
        let res = parser.parse(test_sentence).unwrap();
        assert_eq!(res, ["안녕".to_owned(), "새".to_owned(), "상".to_owned(),]);
    }
}
