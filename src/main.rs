mod scanner;
use crate::scanner::Scanner;
use std::env;
use std::io::{self, Write, BufRead};
use std::fs;
use regex::Regex;

#[macro_use]
extern crate enum_map;

use enum_map::EnumMap;

#[derive(Debug, Enum)]
enum TokenType {
    WS,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
}

fn str (s: &str) -> String {
    s.to_string()
}

fn main() {
    // let mut map = enum_map! {
        // TokenType::WS => str("WS"),
    //     TokenType::LeftParen => str("LeftParen"),
    //     TokenType::RightParen => str("RightParen"),
    //     TokenType::LeftBrace => str("LeftBrace"),
    //     TokenType::RightBrace => str("RightBrace"),
    // };
    let dictionary: Vec<(String, Regex)> = vec![
        // entire nodes
        (str("InlineComment"), Regex::new(r"^//.*").unwrap()),
        (str("WS"), Regex::new(r"^[\t ]+").unwrap()),
        (str("NL"), Regex::new(r"^(?:\n|\n\r)+").unwrap()),
        (str("Keyword"), Regex::new(r"^(?:struct)").unwrap()),
        (str("CapitalIdentifier"), Regex::new(r"^[A-Z][a-zA-Z0-9_-]*").unwrap()),
        (str("Identifier"), Regex::new(r"^[a-z][a-zA-Z0-9_-]*").unwrap()),
        (str("GenericIdentifier"), Regex::new(r"^'[a-z]").unwrap()),
        (str("String"), Regex::new("^\"[^\"]*\"").unwrap()),
        (str("Char"), Regex::new(r"^'[^']'").unwrap()),
        (str("Float"), Regex::new(r"^(?:(?:0|[1-9][0-9]*)f|(?:0|[1-9][0-9]*)\.[0-9]*)").unwrap()),
        (str("Integer"), Regex::new(r"^(?:0|[1-9][0-9]*)").unwrap()),
        // single chars
        (str("LeftParen"), Regex::new(r"^\(").unwrap()),
        (str("RightParen"), Regex::new(r"^\)").unwrap()),
        (str("LeftBrace"), Regex::new(r"^\{").unwrap()),
        (str("RightBrace"), Regex::new(r"^\}").unwrap()),
        (str("LeftBracket"), Regex::new(r"^\[").unwrap()),
        (str("RightBracket"), Regex::new(r"^\]").unwrap()),
        (str("Colon"), Regex::new(r"^:").unwrap()),
        (str("Dot"), Regex::new(r"^\.").unwrap()),
        (str("Equal"), Regex::new(r"^=").unwrap()),
        (str("Dash"), Regex::new(r"^-").unwrap()),
        (str("GreaterThan"), Regex::new(r"^>").unwrap()),
        (str("Hash"), Regex::new(r"^#").unwrap()),
    ];
    let args: Vec<String> = env::args().collect();
    let mut scanner = Scanner::new(dictionary);
    
    if args.len() == 1 {
        run_loop(&mut scanner);
    } else if args.len() == 2 {
        match run_file(args.get(1), &mut scanner) {
            Ok(_) => (),
            Err(e) => println!("{e}")
        }
    } else {
        usage();
    }
}

fn run_loop(scanner: &mut Scanner) {
    loop {
        print!("> ");
        match io::stdout().flush() {
            Ok(_) => (),
            Err(e) => println!("Error: {}", e),
        }

        let mut buffer = String::new();
        let stdin = io::stdin();
        let mut handle = stdin.lock();
        match handle.read_line(&mut buffer) {
            Ok(_) => {
                if buffer.trim().to_string().to_ascii_lowercase() == ":e" {
                    println!("exit");
                    return ();
                }
            },
            Err(e) => println!("Error: {}", e),
        }

        match run(&buffer, scanner) {
            Ok(_) => (),
            Err(e) => println!("Error: {}", e),
        }
    }
}


fn run_file(path_str: Option<&String>, scanner: &mut Scanner) -> Result<(), String> {
    path_str.map(|path| {
        match fs::read_to_string(path) {
            Ok(contents) => {
                run(&contents, scanner)
            },
            Err(e) => Err(format!("Error reading file \"{path}\":\n{e}")),
        }
    }).unwrap_or(Err("No file specified".to_string()))
}

fn run(str: &str, scanner: &mut Scanner) -> Result<(), String> {
    scanner.load(str.to_string());
    let tokens = scanner.scan()?;
    for t in tokens {
        println!("token: {}", t.to_string());
    }
    Ok(())
}

fn usage() {
    println!("Usage: rlx [script]");
}