mod scanner;
mod parser;

use crate::scanner::Scanner;
use crate::parser::Parser;

use std::env;
use std::io::{self, Write, BufRead};
use std::fs;
use regex::Regex;

#[macro_use]
extern crate enum_map;
#[macro_use]
extern crate lazy_static;

#[derive(Debug, Enum, Clone)]
enum TokenType {
    InlineComment,
    WS,
    NL,
    Keyword,
    Identifier,
    CapitalIdentifier,
    GenericIdentifier,
    String,
    Char,
    Float,
    Integer,

    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Colon,
    SemiColon,
    Comma,
    Dot,
    Equal,
    Dash,
    Plus,
    Star,
    Slash,
    GreaterThan,
    Hash,
    EOF
}

lazy_static! {
    static ref INLINE_COMMENT: String = "InlineComment".to_string();
    static ref WS: String = "WS".to_string();
    static ref NL: String = "NL".to_string();
    static ref KEYWORD: String = "Keyword".to_string();
    static ref CAPITAL_IDENTIFIER: String = "CapitalIdentifier".to_string();
    static ref IDENTIFIER: String = "Identifier".to_string();
    static ref GENERIC_IDENTIFIER: String = "GenericIdentifier".to_string();
    static ref STRING: String = "String".to_string();
    static ref CHAR: String = "Char".to_string();
    static ref FLOAT: String = "Float".to_string();
    static ref INTEGER: String = "Integer".to_string();

    static ref LEFT_PAREN: String = "LeftParen".to_string();
    static ref RIGHT_PAREN: String = "RightParen".to_string();
    static ref LEFT_BRACE: String = "LeftBrace".to_string();
    static ref RIGHT_BRACE: String = "RightBrace".to_string();
    static ref LEFT_BRACKET: String = "LeftBracket".to_string();
    static ref RIGHT_BRACKET: String = "RightBracket".to_string();
    static ref COLON: String = "Colon".to_string();
    static ref SEMI_COLON: String = "SemiColon".to_string();
    static ref COMMA: String = "Comma".to_string();
    static ref DOT: String = "Dot".to_string();
    static ref EQUAL: String = "Equal".to_string();
    static ref DASH: String = "Dash".to_string();
    static ref PLUS: String = "Plus".to_string();
    static ref STAR: String = "Star".to_string();
    static ref SLASH: String = "Slash".to_string();
    static ref GREATER_THAN: String = "GreaterThan".to_string();
    static ref HASH: String = "Hash".to_string();
    static ref EOF: String = "EOF".to_string();
}

fn main() {
    let map = enum_map! {
        TokenType::InlineComment => (Regex::new(r"^//.*").unwrap(), INLINE_COMMENT.to_string()),
        TokenType::WS => (Regex::new(r"^[\t ]+").unwrap(), WS.to_string()),
        TokenType::NL => (Regex::new(r"^(?:\n|\n\r)+").unwrap(), NL.to_string()),
        TokenType::Keyword => (Regex::new(r"^(?:struct)").unwrap(), KEYWORD.to_string()),
        TokenType::Identifier => (Regex::new(r"^[a-z][a-zA-Z0-9_-]*").unwrap(), IDENTIFIER.to_string()),
        TokenType::CapitalIdentifier => (Regex::new(r"^[A-Z][a-zA-Z0-9_-]*").unwrap(), CAPITAL_IDENTIFIER.to_string()),
        TokenType::GenericIdentifier => (Regex::new(r"^'[a-z]").unwrap(), GENERIC_IDENTIFIER.to_string()),
        TokenType::String => (Regex::new("^\"[^\"]*\"").unwrap(), STRING.to_string()),
        TokenType::Char => (Regex::new(r"^'[^']'").unwrap(), CHAR.to_string()),
        TokenType::Float => (Regex::new(r"^(?:(?:0|[1-9][0-9]*)f|(?:0|[1-9][0-9]*)\.[0-9]*)").unwrap(), FLOAT.to_string()),
        TokenType::Integer => (Regex::new(r"^(?:0|[1-9][0-9]*)").unwrap(), INTEGER.to_string()),

        TokenType::LeftParen => (Regex::new(r"^\(").unwrap(), LEFT_PAREN.to_string()),
        TokenType::RightParen => (Regex::new(r"^\)").unwrap(), RIGHT_PAREN.to_string()),
        TokenType::LeftBrace => (Regex::new(r"^\{").unwrap(), LEFT_BRACE.to_string()),
        TokenType::RightBrace => (Regex::new(r"^\}").unwrap(), RIGHT_BRACE.to_string()),
        TokenType::LeftBracket => (Regex::new(r"^\[").unwrap(), LEFT_BRACKET.to_string()),
        TokenType::RightBracket => (Regex::new(r"^\]").unwrap(), RIGHT_BRACKET.to_string()),
        TokenType::Colon => (Regex::new(r"^:").unwrap(), COLON.to_string()),
        TokenType::SemiColon => (Regex::new(r"^;").unwrap(), SEMI_COLON.to_string()),
        TokenType::Comma => (Regex::new(r"^,").unwrap(), COMMA.to_string()),
        TokenType::Dot => (Regex::new(r"^\.").unwrap(), DOT.to_string()),
        TokenType::Equal => (Regex::new(r"^=").unwrap(), EQUAL.to_string()),
        TokenType::Dash => (Regex::new(r"^-").unwrap(), DASH.to_string()),
        TokenType::Plus => (Regex::new(r"^\+").unwrap(), PLUS.to_string()),
        TokenType::Star => (Regex::new(r"^\*").unwrap(), STAR.to_string()),
        TokenType::Slash => (Regex::new(r"^/").unwrap(), SLASH.to_string()),
        TokenType::GreaterThan => (Regex::new(r"^>").unwrap(), GREATER_THAN.to_string()),
        TokenType::Hash => (Regex::new(r"^#").unwrap(), HASH.to_string()),
        
        TokenType::EOF => (Regex::new(r"\z").unwrap(), EOF.to_string()),
    };
    let args: Vec<String> = env::args().collect();
    let mut scanner = Scanner::new(map);
    let mut parser = Parser::new(
    "| | | "
    // literal := number | $String
    // binary_op := ($Plus $Dot) | ($Dash $Dot) | Plus | Dash"
    .to_string()
    );
    let g = parser.and_then(|mut p| {
        p.parse()?;
        Ok(())
    });
    println!("\n\n[PARSE-RESULT]:");
    dbg!(g);
    
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

fn run_loop(scanner: &mut Scanner<TokenType>) {
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


fn run_file(path_str: Option<&String>, scanner: &mut Scanner<TokenType>) -> Result<(), String> {
    path_str.map(|path| {
        match fs::read_to_string(path) {
            Ok(contents) => {
                run(&contents, scanner)
            },
            Err(e) => Err(format!("Error reading file \"{path}\":\n{e}")),
        }
    }).unwrap_or(Err("No file specified".to_string()))
}

fn run(str: &str, scanner: &mut Scanner<TokenType>) -> Result<(), String> {
    let tokens = scanner.scan(str.to_string(), TokenType::EOF)?;
    // for t in tokens {
    //     println!("token: {}", t.to_string());
    // }
    Ok(())
}

fn usage() {
    println!("Usage: rlx [script]");
}