use pyo3::prelude::*;

#[derive(Debug, Clone, Copy)]
pub enum ParserKind {
    Khaiii,
    Komoran,
}

impl Default for ParserKind {
    fn default() -> Self {
        ParserKind::Komoran
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Parser {
    parser: ParserKind,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            parser: ParserKind::default(),
        }
    }

    pub fn parse(self, sentence: Sentence) -> PyResult<Vec<(String, String)>> {
        match self.parser {
            ParserKind::Khaiii => self.khaiii_parse(sentence),
            ParserKind::Komoran => self.komoran_parse(sentence),
        }
    }

    fn komoran_parse(self, sentence: Sentence) -> PyResult<Vec<(String, String)>> {
        let res: PyResult<Vec<(String, String)>> = Python::with_gil(|py| {
            let fun = PyModule::from_code(
                py,
                "def parse(*args):
                from konlpy.tag import Komoran
                return Komoran().pos(args[0])",
                "",
                "",
            )?;

            let result = fun
                .getattr("parse")?
                .call1((sentence.sentence,))?
                .extract()?;
            Ok(result)
        });
        Ok(res?)
    }

    fn khaiii_parse(self, sentence: Sentence) -> PyResult<Vec<(String, String)>> {
        let res: PyResult<Vec<Vec<(String, String)>>> = Python::with_gil(|py| {
            let fun = PyModule::from_code(
                py,
                "def parse(*args):
                from khaiii import KhaiiiApi
                data = KhaiiiApi().analyze(args[0])

                word_list = list()
                words = [word.morphs for word in data]
                for word in words:
                    word = [(morphs.lex, morphs.tag) for morphs in word]
                    word_list.append(word)
                return word_list",
                "",
                "",
            )?;

            let result = fun
                .getattr("parse")?
                .call1((sentence.sentence,))?
                .extract()?;

            Ok(result)
        });
        //let res = res?;
        //temporary fix to get code to run. TODO need to do actual parsing
        let mut words: Vec<(String, String)> = Vec::new();

        res?.into_iter().for_each(|sent| {
            println!("{:?}", sent);
            words.push(sent[0].clone());
        });

        Ok(words)
    }

    //slient error being handled here MAKE SURE TO FIX
    pub fn change_parser(mut self, parser: ParserKind) -> Self {
        let flag = Self::check_khaiii_install();
        let parser = match (parser, flag) {
            (ParserKind::Khaiii, true) => ParserKind::Khaiii,
            (ParserKind::Khaiii, false) => ParserKind::Komoran,
            (_, _) => ParserKind::Komoran,
        };
        self.parser = parser;
        self
    }

    pub fn check_khaiii_install() -> bool {
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
        println!("Khaiii installed?: {}", res.as_ref().unwrap());
        res.unwrap()
    }

    pub fn parser_type(&self) -> ParserKind {
        println!("{:?}", self.parser);
        self.parser
    }
}

#[derive(Debug, Clone)]
pub struct Sentence {
    sentence: String,
}

impl Sentence {
    pub fn new(sentence: String) -> Self {
        Self { sentence }
    }

    pub fn parse(self, parser: &Parser) -> PyResult<Vec<(String, String)>> {
        //do I want to handle match statement here or in Parser Struct
        match parser.parser {
            ParserKind::Khaiii => parser.khaiii_parse(self),
            ParserKind::Komoran => parser.komoran_parse(self),
        }
        //passes logic to Parser Struct
        //parser.parse(self)
    }
}
