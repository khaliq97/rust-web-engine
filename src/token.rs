#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen, RIGHT_PAREN, LEFT_BRACE, RIGHT_BRACE,
    COMMA, DOT, MINUS, PLUS, SEMICOLON, SLASH, STAR,
    BITWISE_NOT, COLON,

    // One or two character tokens.
    BANG, BANG_EQUAL,
    EQUAL, EQUAL_EQUAL,
    GREATER, GREATER_EQUAL,
    LESS, LESS_EQUAL,

    // Literals.
    IDENTIFIER, STRING, NUMBER,

    // Reserved keywords.
    // https://tc39.es/ecma262/#prod-ReservedWord
    CLASS, ELSE, FALSE, FOR, IF, NULL,
    RETURN, SUPER, THIS, TRUE, VAR, WHILE,
    AWAIT, BREAK, CASE, CATCH, CONST, CONTINUE, DEBUGGER,
    DEFAULT, DELETE, DO, ENUM, EXPORT, EXTENDS, FINALLY,
    FUNCTION, IMPORT, IN, INSTANCEOF, NEW, SWITCH,
    THROW, TRY, TYPEOF, VOID, WITH, YIELD,

    EOF
}

// https://tc39.es/ecma262/#prod-Literal
#[derive(Clone)]
#[derive(Debug)]
pub enum Literal {
    String(String),
    Numeric(f64),
    Boolean(bool),
    Null()
}

#[derive(Clone)]
#[derive(Debug)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Option<Literal>,
    pub line: usize
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, literal: Option<Literal>, line: usize) -> Token {
        Token { token_type, lexeme, literal, line }
    }

    pub fn to_string(&self) -> String {
        return match &self.literal {
            Some(literal) => {
                format!("{:?} {} {:?}", self.token_type, self.lexeme, literal)
            },
            None => {
                format!("{:?} {}", self.token_type, self.lexeme)
            }
        }
    }
}