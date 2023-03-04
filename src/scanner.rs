use std::{ops::Range, fmt::Debug};
use regex::Regex;
use enum_map::{Enum, EnumMap, EnumArray};

type Label = String;

#[derive(Debug)]
pub struct Scanner<T> where T: Clone + Debug + Enum + EnumArray<(Regex, String)> {
    contents: String,
    tokens: Vec<Token<T>>,
    dictionary: EnumMap<T, (Regex, String)>,
    line: u64,
    col: u64,
    cursor: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token<T> {
    pub token_type: T,
    pub value: String,
    pub line: u64,
    pub col: u64,
}

impl<T> Scanner<T> where T: Clone + Debug + Enum + EnumArray<(Regex, String)> {
    pub fn new(dictionary: EnumMap<T, (Regex, String)>) -> Scanner<T> {
        Scanner {
            contents: String::new(),
            tokens: Vec::new(),
            dictionary,
            line: 1,
            col: 1,
            cursor: 0,
        }
    }

    pub fn scan(&mut self, contents: String, defaut_type: T) -> Result<Vec<Token<T>>, String> {
        self.contents = contents;
        while !self.is_eof() {
            self.match_with_dic()
                .and_then(|(label, value)| {
                    Ok(self.push_token(label, value))
                })?;
        }
        let eof = Token {
            token_type: defaut_type,
            value: "".to_string(),
            line: self.line,
            col: self.col,
        };
        self.tokens.push(eof);
        Ok(self.tokens.clone())
    }

    fn is_eof(&self) -> bool {
        self.cursor >= self.contents.len() as u64
    }

    fn match_with_dic(&mut self) -> Result<(T, String), String> {
        for (type_, (regex, label)) in self.dictionary.iter() {
            let m = regex.find(&self.contents[self.cursor as usize..]);
            if m.is_some() {
                let m = m.unwrap();
                let r = self.translate_range_usize(m.start(), m.end()); 
                let val = self.contents[r].to_string();
                return Ok((type_, val));
            }
        }
        let char = self.contents.chars().nth(self.cursor as usize).unwrap_or('\0');
        Err(format!(
            "Unexpected character '{}' at line {} col {}",
            Scanner::<T>::safe_char(char),
            self.line,
            self.col
        ))
    }

    fn push_token(&mut self, t_type: T, value: String) {
        let l = value.len() as u64;
        let lines = Scanner::<T>::count_char(&value, '\n');
        let t = Token {
            token_type: t_type,
            value: if lines > 0 { "".to_string() } else { value },
            line: self.line,
            col: self.col,
        };
        self.tokens.push(t);
        self.cursor += l;
        self.line += lines;
        self.col = if lines > 0 { 0 } else { self.col + l };
    }

    fn safe_char(c: char) -> String {
        match c {
            '\0' => "\\0".to_string(),
            '\n' => "\\n".to_string(),
            '\t' => "\\t".to_string(),
            '\r' => "\\r".to_string(),
            _ => c.to_string()
        }
    }

    fn count_char(str: &String, char: char) -> u64 {
        let mut counter = 0;
        for c in str.chars() {
            if c == char {
                counter += 1;
            }
        }
        counter
    }

    fn translate_range_usize(&self, start: usize, end: usize) -> Range<usize> {
        let start = self.cursor as usize + start;
        let end = self.cursor as usize + end;
        start..end
    }
}

impl<T> Token<T> where T: Debug {
    pub fn to_string(&self) -> String {
        format!("{:?} {} - {}:{}", self.token_type, self.value, self.line, self.col)
    }
}
