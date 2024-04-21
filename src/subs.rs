use std::fs;

pub struct MovieTime {
    mili: u32,
}

impl MovieTime {
    fn from_str(timming: &str) -> Self {
        Self {
            mili: timming[9..12].parse::<u32>().unwrap() // milisecond
            + 1000 * (timming[6..8].parse::<u32>().unwrap() // second
            + 60 * (timming[3..5].parse::<u32>().unwrap() // minute
            + 60 * timming[0..2].parse::<u32>().unwrap())) // hour
        }
    }

    pub fn to_string(&self) -> String {
        let mut dividend_time = self.mili;
        let miliseconds = dividend_time % 1000;
        dividend_time /= 1000;
        let seconds = dividend_time % 60;
        dividend_time /= 60;
        let minutes = dividend_time % 60;
        dividend_time /= 60;
        format!("{:02}:{:02}:{:02},{:03}", dividend_time, minutes, seconds, miliseconds)
    }
}

pub struct Subtitle {
    index: u16,
    pub start: MovieTime,
    pub stop: MovieTime,
    pub text: String,
}


pub fn load_subtitles(file_path: &String) -> Vec<Subtitle> {
    let mut index_buffer = "";
    let mut timming_buffer = "";
    let mut text_buffer = String::from("");
    let mut subtitles: Vec<Subtitle> = Vec::new();

    let contents = fs::read_to_string(file_path).expect("Failure at reading file");

    for line in contents.lines() {
        if line != "" {
            if index_buffer == "" {
                index_buffer = line;
            } else if timming_buffer == "" {
                timming_buffer = line;
            } else {
                if text_buffer.len() > 0 {
                    text_buffer.push_str("\n");
                }
                text_buffer.push_str(line);
            }
        } else {
            let timmings: Vec<&str> = timming_buffer.split(" --> ").collect();
            subtitles.push(Subtitle {
                index: index_buffer
                .parse()
                .expect("Error in extracting subtitle index"),
                           start: MovieTime::from_str(timmings[0]),
                           stop: MovieTime::from_str(timmings[1]),
                           text: text_buffer,
            });
            index_buffer = "";
            timming_buffer = "";
            text_buffer = String::from("");
        }
    }
    subtitles.sort_by(|a,b| a.index.partial_cmp(&b.index).unwrap());

    return subtitles;
}
