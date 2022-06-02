use pyo3::prelude::*;
//use pyo3::types::PyList;
//use pyo3::types::PyTuple;

#[derive(Debug)]
pub struct Parser {
    pub parser: String,
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            parser: "khaiii".to_owned(),
        }
    }

    pub fn parse(&self, sentence: String) -> PyResult<Vec<Vec<(String, String)>>> {
        let res: PyResult<Vec<Vec<(String, String)>>> = Python::with_gil(|py| {
            let fun = PyModule::from_code(
                py,
                "def parse(*args, **kwargs):
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
                .call1((sentence,))?
                .extract::<Vec<Vec<(String, String)>>>()?;

            Ok(result)
        });
        Ok(res?)
    }
}
