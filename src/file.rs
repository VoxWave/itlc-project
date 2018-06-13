use std::io::{BufReader, BufRead};
use std::fs::File;
use common::Source;

fn load_file_as_char_source() -> CharFile {
    
}

pub struct CharFile {
    line: String,
    file: BufReader<File>,
}

impl CharFile {
    pub fn new(path: &str) -> Self {
        let mut file = BufReader::new(File::open(path).unwrap());
        CharFile {
            line: String::new(),
            file,
        }
    }
}

impl Source<char> for CharFile {
    fn take(&mut self) -> Option<char> {
        match self.line.pop() {
            None => {
                match self.file.read_line(&mut self.line).unwrap() {
                    0 => None,
                    _ => {
                        self.line = self.line.chars().rev().collect();
                        self.line.pop()
                    },
                }
            },
            c => c,
        }
    }
}