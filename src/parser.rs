use crate::scanner::*;
use regex::Regex;

use enum_map::EnumMap;

type TToken = Token<TokenType>;
type Map = EnumMap<TokenType, (Regex, String)>;

#[derive(Debug)]
pub struct Parser {
  scanner: Scanner<TokenType>,
  internal_tokens: Vec<TToken>,
  map: EnumMap<TokenType, (Regex, String)>,
  cursor: u64,
}

#[derive(Enum, Debug, Clone, PartialEq)]
enum TokenType {
    InlineComment,
    Identifier,
    WS,
    NL,
    Assign,
    Var,
    Or,
    LeftParen,
    RightParen,
    Star,
    Plus,
    Question,
    EOF,
}

#[derive(Debug)]
pub struct AssignNode {
    id: TToken,
    value: Expression,
}

#[derive(Debug)]
enum Expression {
    Or(Box<Expression>, Box<Expression>),
    Sequence(Vec<Expression>),
    Many(Vec<Expression>),
    ManyOne(Vec<Expression>),
    Optional(Box<Expression>),
    Group(Box<Expression>),
    Assign(Box<AssignNode>),
    Val(TToken),
}


lazy_static! {
    static ref INLINE_COMMENT: String = "InlineComment".to_string();
    static ref IDENTIFIER: String = "Identifier".to_string();
    static ref WS: String = "WS".to_string();
    static ref NL: String = "NL".to_string();
    static ref ASSIGN: String = "Assign".to_string();
    static ref VAR: String = "Var".to_string();
    static ref OR: String = "Or".to_string();
    static ref LEFT_PAREN: String = "LeftParen".to_string();
    static ref RIGHT_PAREN: String = "RightParen".to_string();
    static ref STAR: String = "Star".to_string();
    static ref PLUS: String = "Plus".to_string();
    static ref QUESTION: String = "Question".to_string();
    static ref EOF: String = "EOF".to_string();
}

impl Parser {
    pub fn new(rules: String) -> Result<Parser, String> {
        let map = enum_map! {
            TokenType::InlineComment => (Regex::new(r"^//.*").unwrap(), INLINE_COMMENT.to_string()),
            TokenType::Identifier => (Regex::new(r"^[a-z-A-Z_][a-zA-Z0-9_-]*").unwrap(), IDENTIFIER.to_string()),
            TokenType::WS => (Regex::new(r"^[\t ]+").unwrap(), WS.to_string()),
            TokenType::NL => (Regex::new(r"^(?:\n|\n\r)+").unwrap(), NL.to_string()),
            TokenType::Assign => (Regex::new(r"^:=").unwrap(), ASSIGN.to_string()),
            TokenType::Var => (Regex::new(r"^\$[a-z-A-Z][a-zA-Z0-9_-]*").unwrap(), VAR.to_string()),
            TokenType::Or => (Regex::new(r"^\|").unwrap(), OR.to_string()),
            TokenType::LeftParen => (Regex::new(r"^\(").unwrap(), LEFT_PAREN.to_string()),
            TokenType::RightParen => (Regex::new(r"^\)").unwrap(), RIGHT_PAREN.to_string()),
            TokenType::Star => (Regex::new(r"^\*").unwrap(), STAR.to_string()),
            TokenType::Plus => (Regex::new(r"^\+").unwrap(), PLUS.to_string()),
            TokenType::Question => (Regex::new(r"^\?").unwrap(), QUESTION.to_string()),
            TokenType::EOF => (Regex::new(r"^\z").unwrap(), EOF.to_string()),
        };
        let mut scanner = Scanner::new(map.clone());
        let tokens = scanner.scan(rules, TokenType::EOF);
        match tokens {
            Ok(tokens) => Ok(Parser {
                scanner,
                internal_tokens: tokens,
                map,
                cursor: 0,
            }),
            Err(err) => Err(err),
        }
    }

    pub fn parse(&mut self) -> Result<(), String> {
        while !self.is_eof() {
            let node = self.sequence(vec![TokenType::Or, TokenType::WS])?;
            print!("\n\n[NODE]\n");
            dbg!(node);
        }
        Ok(())
    }

    // fn parse_assign(&mut self) -> Result<(AssignNode, u8), String> {
    //     self.     
    // }

    fn parse_terminal(&mut self) -> Result<Expression, String> {
        let token = self.choice(vec![TokenType::Var, TokenType::Identifier])?;
        match token.token_type {
            TokenType::Identifier => Ok(Expression::Val(token)),
            TokenType::Var => Ok(Expression::Val(token)),
            _ => Err(format!("Unexpected token: {:?}", token)),
        }
    }

    fn or(&mut self, a: TokenType, b: TokenType) -> Result<TToken, String> {
        let t = &self.internal_tokens;
        let c = self.cursor;
        let m = &self.map;
        let token_a = Parser::consume(a, t, c, m);
        if token_a.is_ok() {
            self.cursor += 1;
            return token_a;
        }
        let token_b = Parser::consume(b, t, c + 1, m);
        if token_b.is_ok() {
            self.cursor += 1;
        }
        token_b
    }

    fn and(&mut self, a: TokenType, b: TokenType) -> Result<(TToken, TToken), String> {
        let t = &self.internal_tokens;
        let c = self.cursor;
        let m = &self.map;
        let token_a = Parser::consume(a, t, c, m);
        match token_a {
            Err(err) => Err(err),
            Ok(token_a) => {
                let token_b = Parser::consume(b, t, c + 1, m);
                match token_b {
                    Err(err) => Err(err),
                    Ok(token_b) => {
                        self.cursor = c + 2;
                        Ok((token_a, token_b))
                    },
                }
            }
        }
    }

    fn choice(&mut self, opts: Vec<TokenType>) -> Result<Token<TokenType>, String> {
        if opts.len() < 1 { return Err("ParserError [choice]: Not enough options".to_string()); }
        if opts.len() == 1 { return Parser::consume(opts[0].clone(), &self.internal_tokens, self.cursor, &self.map)}
        let mut i = 0;
        let mut j = 1;
        while j < opts.len() {
            let t_a = opts[i].clone();
            let t_b = opts[j].clone();
            let r = self.or(t_a, t_b);
            match r {
                Ok(token) => {
                    self.cursor += 1;
                    return Ok(token)
                },
                _ => { i += 1; j += 1; }
            }            
        }
        let opts_str = opts.iter()
            .map(|x| self.find_token_label(x))
            .collect::<Vec<String>>()
            .join(" or ");
        let d = Token {
            token_type: TokenType::EOF,
            value: "EOF".to_string(),
            line: 0,
            col: 0,
        };
        let current = self.current().unwrap_or(&d);
        Err(format!("Expecting: {}, found: {:?}", opts_str, current.token_type))
    }

    fn sequence(&mut self, opts: Vec<TokenType>) -> Result<Vec<TToken>, String> {
        let mut tokens: Vec<TToken> = Vec::new();
        if opts.len() < 2 { return Err("ParserError [sequence]: Not enough options".to_string()); }
        let mut i = 0;
        let mut j = 1;
        while j < opts.len() {
            let t_a = opts[i].clone();
            let t_b = opts[j].clone();
            let r = self.and(t_a, t_b);
            match r {
                Ok((token_a, token_b)) => {
                    tokens.push(token_a);
                    tokens.push(token_b);
                    i += 1; j += 1;
                },
                Err(_) => {
                    let opts_str = opts.iter()
                        .map(|x| self.find_token_label(x))
                        .collect::<Vec<String>>()
                        .join(" followed by ");
                    let d = Token {
                        token_type: TokenType::EOF,
                        value: "EOF".to_string(),
                        line: 0,
                        col: 0,
                    };
                    let current = self.current().unwrap_or(&d);
                    return Err(format!("Expecting: {}, found: {:?}", opts_str, current.token_type))
                }
            }
        }
        Ok(tokens)
    }

    fn find_token_label(&self, t_type: &TokenType) -> String {
        let (_, label) = &self.map[t_type.clone()];
        label.clone()
    }

    fn consume(
        token_type: TokenType,
        tokens: &Vec<Token<TokenType>>,
        cursor: u64,
        map: &Map
    ) -> Result<Token<TokenType>, String> {
        let token = tokens.get(cursor as usize);
        if token.is_none() {
            return Err("Unexpected EOF".to_string());
        }
        let token = token.unwrap();
        let (_, label) = (*map)[token_type.clone()].clone();
        if token.token_type != token_type {
            return Err(format!("Expected {}, got {:?}", label, token.token_type));
        }
        Ok(token.clone())
    }

    fn is_eof(&mut self) -> bool {
        self.cursor >= self.internal_tokens.len() as u64
    }

    fn current(&self) -> Option<&Token<TokenType>> {
        self.internal_tokens.get(self.cursor as usize)
    }

}