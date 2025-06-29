use std::collections::HashMap;
use super::token::{Token, TokenType, Literal};

pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    reserved_keywords: HashMap<String, TokenType>
}

impl Scanner {
    pub fn new(source: String) -> Scanner {
        // https://tc39.es/ecma262/#prod-ReservedWord
        let reserved_keywords: HashMap<String, TokenType> = [
            ("await".to_string(), TokenType::AWAIT),
            ("break".to_string(), TokenType::BREAK),
            ("case".to_string(), TokenType::CASE),
            ("catch".to_string(), TokenType::CATCH),
            ("class".to_string(), TokenType::CLASS),
            ("const".to_string(), TokenType::CONST),
            ("continue".to_string(), TokenType::CONTINUE),
            ("debugger".to_string(), TokenType::DEBUGGER),
            ("default".to_string(), TokenType::DEFAULT),
            ("delete".to_string(), TokenType::DELETE),
            ("do".to_string(), TokenType::DO),
            ("else".to_string(), TokenType::ELSE),
            ("enum".to_string(), TokenType::ENUM),
            ("export".to_string(), TokenType::EXPORT),
            ("extends".to_string(), TokenType::EXTENDS),
            ("false".to_string(), TokenType::FALSE),
            ("finally".to_string(), TokenType::FINALLY),
            ("for".to_string(), TokenType::FOR),
            ("function".to_string(), TokenType::FUNCTION),
            ("if".to_string(), TokenType::IF),
            ("import".to_string(), TokenType::IMPORT),
            ("in".to_string(), TokenType::IN),
            ("instanceof".to_string(), TokenType::INSTANCEOF),
            ("new".to_string(), TokenType::NEW),
            ("null".to_string(), TokenType::NULL),
            ("return".to_string(), TokenType::RETURN),
            ("super".to_string(), TokenType::SUPER),
            ("switch".to_string(), TokenType::SWITCH),
            ("this".to_string(), TokenType::THIS),
            ("throw".to_string(), TokenType::THROW),
            ("true".to_string(), TokenType::TRUE),
            ("try".to_string(), TokenType::TRY),
            ("typeof".to_string(), TokenType::TYPEOF),
            ("var".to_string(), TokenType::VAR),
            ("void".to_string(), TokenType::VOID),
            ("while".to_string(), TokenType::WHILE),
            ("with".to_string(), TokenType::WITH),
            ("yield".to_string(), TokenType::YIELD),
        ].iter().cloned().collect();

        Scanner { 
            source, 
            tokens: Vec::new(), 
            start: 0, 
            current: 0, 
            line: 0,
            reserved_keywords 
        }
    }

    pub fn scan_tokens(&mut self) -> &Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens.push(Token::new(TokenType::EOF, String::from(""), None, self.line));

        return &self.tokens;
    }

    fn is_at_end(&self) -> bool {
        return self.current >= self.source.len();
    }

    fn scan_token(&mut self) {
        let c = &self.advance();
        match c {
            '(' => {
                self.add_token(TokenType::LeftParen, None);
            },
            ')' => {
                self.add_token(TokenType::RIGHT_PAREN, None);
            },
            '{' => {
                self.add_token(TokenType::LEFT_BRACE, None);
            },
            '}' => {
                self.add_token(TokenType::RIGHT_BRACE, None);
            },
            ',' => {
                self.add_token(TokenType::COMMA, None);
            },
            '.' => {
                self.add_token(TokenType::DOT, None);
            },
            '-' => {
                self.add_token(TokenType::MINUS, None);
            },
            '+' => {
                self.add_token(TokenType::PLUS, None);
            },
            ';' => {
                self.add_token(TokenType::SEMICOLON, None);
            },
            '*' => {
                self.add_token(TokenType::STAR, None);
            },
            '~' => {
                self.add_token(TokenType::BITWISE_NOT, None);
            },
            ':' => self.add_token(TokenType::COLON, None),
            '!' => {
                if self.match_token('=') {
                    self.add_token(TokenType::BANG_EQUAL, None);
                } else {
                    self.add_token(TokenType::BANG, None);
                }
            },
            '=' => {
                if self.match_token('=') {
                    self.add_token(TokenType::EQUAL_EQUAL, None);
                } else {
                    self.add_token(TokenType::EQUAL, None);
                }
            },
            '<' => {
                if self.match_token('=') {
                    self.add_token(TokenType::LESS_EQUAL, None);
                } else {
                    self.add_token(TokenType::LESS, None);
                }
            },
            '>' => {
                if self.match_token('=') {
                    self.add_token(TokenType::GREATER_EQUAL, None);
                } else {
                    self.add_token(TokenType::GREATER, None);
                }
            },

            '/' => {
                if self.match_token('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::SLASH, None);
                }
            },
            ' ' | '\r' | '\t' => {
                // Ignore whitespace.
            },
            '\n' => {
                 self.line += 1;
            },
            '"' => { self.string() },
            _ => {
                if self.is_digit(*c) {
                    self.number();
                } else if self.is_alpha_numeric(*c) {
                    self.identifier();
                } else {
                    Self::error(self.line, "Unexpected character: ".to_string() + &c.to_string());
                }
            }
        }
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            Self::error(self.line, "Unterminated string.".to_string());
            return;
        }

        // The closing ".

        self.advance();

        // Trim the surrounding quotes.
        let value = self.source[self.start + 1..self.current - 1].to_string();
        self.add_token(TokenType::STRING, Some(Literal::String(value)));
    }

    fn number(&mut self) {
        while self.is_digit(self.peek()) {
            self.advance();
        }

        // Look for a fractional part
        if self.peek() == '.' && self.is_digit(self.peek_next()) {
            // Consume the "."
            self.advance();
            while self.is_digit(self.peek()) {
                self.advance();
            }
        }

        let chars: Vec<char> = self.source.chars().collect();
        self.add_token(TokenType::NUMBER, Option::from(Literal::Numeric(self.source[self.start..self.current].parse::<f64>().unwrap())));
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0';
        }
        let chars: Vec<char> = self.source.chars().collect();
        return chars[self.current + 1];
    }

    fn peek(&self) -> char {
        let chars: Vec<char> = self.source.chars().collect();
        if self.is_at_end() {
            return '\0';
        }
        return chars[self.current];
    }

    fn advance(&mut self) -> char {
        let chars: Vec<char> = self.source.chars().collect();
        let current_char = chars[self.current];
        self.current = self.current + 1;
        return current_char;
    }

    fn add_token(&mut self, token_type: TokenType, literal: Option<Literal>) {
        match literal {
            Some(literal) => {
                let text: String = self.source[self.start..self.current].to_string();
                self.tokens.push(Token::new(token_type, text, Option::from(literal), self.line));
            },
            None => {
                let text: String = self.source[self.start..self.current].to_string();
                self.tokens.push(Token::new(token_type, text, None, self.line));
            }
        }
    }

    fn match_token(&mut self, expected: char) -> bool {
        let chars: Vec<char> = self.source.chars().collect();
        if self.is_at_end() || chars[self.current] != expected {
            return false;
        }
        self.current += 1;
        return true;
    }

    fn is_digit(&self, c: char) -> bool {
        return c >= '0' && c <= '9';
    }

    fn identifier(&mut self) {
        while self.is_alpha_numeric(self.peek()) {
            self.advance();
        }

        let text = self.source[self.start..self.current].to_string();
        let token_type = self.reserved_keywords.get(&text).unwrap_or(&TokenType::IDENTIFIER).clone();

        self.add_token(token_type, None);
    }

    fn is_alpha(&self, c: char) -> bool {
        return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_';
    }

    fn is_alpha_numeric(&self, c: char) -> bool {
        return self.is_alpha(c) || self.is_digit(c);
    }

    fn error(line: usize, message: String) {
        println!("Error on line {}: {}", line, message);
    }
}
