use std::io::{BufReader, Read};
use std::fs::File;

pub struct Lexer { 
    position: usize,
    tokens: Vec<u8>,
    pub tokens_length: usize
}

impl Lexer { 
    pub fn new(source: String) -> Self { 

        let position = 0;
        
        let file = File::open(source.clone()).expect("File could not opened!");
        let mut reader = BufReader::new(file);

        let mut tokens = Vec::new();

        reader.read_to_end(&mut tokens).expect("File could not be read!");

        let tokens_length = tokens.len();

        Self { position, tokens, tokens_length }
    }

    pub fn peek(&mut self) -> Option<char> {
        if self.position != self.tokens_length { 
            let peeked_character = self.tokens[self.position] as char;
            return Some(peeked_character);
        } else { 
            None
        }
      
    }

    pub fn peekNext(&mut self) -> Option<char> { 
        if self.position != self.tokens_length { 
            let peeked_character = self.tokens[self.position + 1] as char;
            return Some(peeked_character);
        } else { 
            None
        }
    }

    pub fn advance(&mut self) { 
        self.position += 1;
    }

    pub fn rewindAndPeek(&mut self, amount: usize) -> Option<char> { 
        if self.position != self.tokens_length { 
            let peeked_character = self.tokens[self.position - amount] as char;
            return Some(peeked_character);
        } else { 
            None
        }
    }

    pub fn rewind(&mut self, amount: usize) { 
        self.position -= amount;
    }

    pub fn previous(&mut self) -> Option<char> {
        if self.position != self.tokens_length { 
            let peeked_character = self.tokens[self.position - 1] as char;
            return Some(peeked_character);
        } else { 
            None
        }
      
    }
}