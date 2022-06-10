use csv::ReaderBuilder;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::iter::zip;

pub fn csv_parse(path: String) -> Result<Vec<HashMap<String, String>>, Box<dyn Error>> {
    let tags: Vec<String> = vec!["tag", "sq_marker", "audio", "picture", "tl_subs", "nl_subs"]
        .into_iter()
        .map(String::from)
        .collect();

    let csv_data = fs::read_to_string(path)?;
    let mut csv_res: Vec<HashMap<String, String>> = Vec::new();
    let rdr = ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b'\t')
        .from_reader(csv_data.as_bytes());

    for result in rdr.into_records() {
        let result: Vec<String> = result?.iter().map(String::from).collect();
        let iter = zip(tags.to_owned(), result);
        let res: HashMap<String, String> = HashMap::from_iter(iter);
        //println!("{res:#?}");
        csv_res.push(res);
    }
    Ok(csv_res)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_csv_parse() {
        let csv_data = csv_parse("resources/test.tsv".into()).unwrap();
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
        assert_eq!(csv_data[0], test_data_first);
        assert_eq!(csv_data[4], test_data_last);
    }
}
