use std::error::Error;

#[derive(Debug, Default)]
pub struct Srt {
    pub subtitles: Vec<Subtitle>
}

#[derive(Debug)]
pub struct Subtitle {
    id: isize,
    time_start: SrtTime,
    time_end: SrtTime,
    text: String,
}

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct SrtTime {
    hr: isize,
    min: isize,
    sec: isize,
    ms: isize
}

impl Srt {
    // pub fn retime(&mut self) {
    //
    //     let mut prev_time_start = SrtTime::default();
    //     for subtitle in self.subtitles.iter_mut().rev() {
    //         if prev_time_start != SrtTime::default(){
    //             subtitle.time_end = prev_time_start;
    //         }
    //         prev_time_start = subtitle.time_start; 
    //     }
    //     
    // }

    pub fn build(self) -> String {
        let mut output: String = String::default();
        for subtitle in self.subtitles {
            let time = format!("{} --> {}",subtitle.time_start.render(), subtitle.time_end.render());
            let sub = format!("{}\r\n{}\r\n{}\r\n\r\n", subtitle.id, time, subtitle.text);
            output.push_str(&sub);
        }

        output.trim().to_string()
    }
}

impl Subtitle {
    pub fn parse(subtitle_raw: String) -> Result<Subtitle, Box<dyn Error>> {

        let subtitle: Vec<String> = subtitle_raw.split_inclusive("\r\n")
        .map(|n| n.to_string())
        .collect();

        let id = subtitle[0].trim().parse::<isize>()?;

        let time: Vec<&str> = subtitle[1].trim().split(" --> ").collect();
        let time_start = SrtTime::parse(time[0])?;
        let time_end = SrtTime::parse(time[1])?;

        let texts: Vec<String> = subtitle[2..].into();
        let text: String = texts.into_iter()
            .collect::<String>()
            .trim()
            .to_string();

        Ok(Subtitle {
            id,
            time_start,
            time_end,
            text
        })
    }
}

impl SrtTime {

    pub fn new(hr: isize, min: isize, sec: isize, ms: isize) -> SrtTime {
        SrtTime {
            hr,
            min,
            sec,
            ms
        }
    }

    pub fn parse(time: &str) -> Result<SrtTime, Box<dyn Error>> {
        let time_raw: Vec<&str> = time.split(':').collect();
        let time_sec_raw: Vec<&str> = time_raw[2].split(',').collect();

        Ok(SrtTime {
            hr: time_raw[0].parse::<isize>()?,
            min: time_raw[1].parse::<isize>()?,
            sec:time_sec_raw[0].parse::<isize>()?,
            ms:time_sec_raw[1].parse::<isize>()?,
        })
    }

    pub fn render(self) -> String {
        format!("{:02}:{:02}:{:02},{:03}", self.hr, self.min, self.sec, self.ms)
    }
}

pub fn srt_parser(data: String) -> Result<Srt, Box<dyn Error>> {
    let subtitles_raw: Vec<String> = data.split("\r\n\r\n")
        .map(|n| n.to_string())
        .collect();

    let mut subtitles: Vec<Subtitle> = vec!();
    for subtitle_raw in subtitles_raw {
        let subtitle = Subtitle::parse(subtitle_raw)?;
        subtitles.push(subtitle);
    }

    Ok(Srt{ subtitles })
}


