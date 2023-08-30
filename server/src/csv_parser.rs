use csv::ReaderBuilder;
use std::collections::HashMap;
use std::error::Error;
use std::iter::zip;
use std::option::Option;

#[derive(Debug)]
pub struct CsvData {
    pub data: Vec<HashMap<String, String>>,
    index: usize,
    size: usize,
    next_state: bool,
    prev_state: bool,
}

impl CsvData {
    fn new(data: Vec<HashMap<String, String>>) -> CsvData {
        //let index = 0;
        //let next_state = true;
        //let prev_state = false;
        let size = data.len();
        Self {
            data,
            index: 0,
            size,
            next_state: true,
            prev_state: false,
        }
    }

    /*
    * I Didn't implement this next function as an Iterator type because the next function doesn't
    * behave as you would expect an iterator on a Vec would. The first return value next on normal lists
    * turned into Iterators is the first value in the Vec but this returns the second leading to
    * wierd behavior
    */
    
    pub fn next_val(&mut self) -> Option<HashMap<String, String>> {
        if !self.next_state {
           return None
        }
        self.index += 1;

        if (self.index + 1) == self.size {
            self.next_state = false;
        }
        //for the love of the memory efficiency gods fix this unholy copying owing atrocity
        Some(self.data[self.index].clone())

    }


    pub fn prev_val(&mut self) -> Option<HashMap<String, String>> {
        if !self.prev_state {
            return None
        }
        self.index -=1;

        if self.index == 0 {
            self.prev_state = false;
        }
        Some(self.data[self.index].clone())
    }

    pub fn current_val(&mut self) -> HashMap<String, String> {
        self.data[self.index].clone()
    }

}

pub fn csv_parse(stream_data: &str) -> Result<CsvData, Box<dyn Error + Send + Sync>> {
    let tags: Vec<String> = vec!["tag", "sq_marker", "audio", "picture", "tl_subs", "nl_subs"]
        .into_iter()
        .map(String::from)
        .collect();

    //let csv_data = stream_data.to_owned();
    let mut csv_res: Vec<HashMap<String, String>> = Vec::new();
    let rdr = ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b'\t')
        .from_reader(stream_data.as_bytes());

    for result in rdr.into_records() {
        let result: Vec<String> = result?.iter().map(String::from).collect();
        let iter = zip(tags.to_owned(), result);
        let res: HashMap<String, String> = HashMap::from_iter(iter);
        //println!("{res:#?}");
        csv_res.push(res);
    }
    Ok(CsvData::new(csv_res))
    //Ok(csv_res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    #[test]
    fn test_csv_parse() {
        let file_data = fs::read_to_string("../resources/test.tsv").unwrap();
        let csv_data = csv_parse(&file_data).unwrap();
        let test_data_first: HashMap<String, String> = HashMap::from([
            ("tag".into(), "bite_sisters_10".into()),
            ("sq_marker".into(), "10_001_0.00.00.431".into()),
            (
                "audio".into(),
                "[sound:bite_sisters_10_0.00.00.431-0.00.04.196.mp3]".into(),
            ),
            (
                "picture".into(),
                "<img src=\"bite_sisters_10_0.00.02.314.jpg\">".into(),
            ),
            (
                "tl_subs".into(),
                "[\"운명의 기복은 친구의 신뢰를 시험한다\" - 키케로]".into(),
            ),
            (
                "nl_subs".into(),
                "[\"The shifts of Fortune test the reliability of friends\" - Cicero]".into(),
            ),
        ]);

        let test_data_last: HashMap<String, String> = HashMap::from([
            ("tag".into(), "bite_sisters_10".into()),
            ("sq_marker".into(), "10_005_0.00.17.133".into()),
            (
                "audio".into(),
                "[sound:bite_sisters_10_0.00.17.133-0.00.20.291.mp3]".into(),
            ),
            (
                "picture".into(),
                "<img src=\"bite_sisters_10_0.00.18.712.jpg\">".into(),
            ),
            ("tl_subs".into(), "수없이 닥쳐오다 지들도 지쳤는지".into()),
            (
                "nl_subs".into(),
                "they must have been exhausted themselves".into(),
            ),
        ]);
        //only testing first and last hashmap because clutter
        assert_eq!(csv_data.data[0], test_data_first);
        assert_eq!(csv_data.data[4], test_data_last);
    }
}
