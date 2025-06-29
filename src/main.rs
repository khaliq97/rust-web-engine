use std::{env, borrow::Borrow};
use std::ops::Deref;
use web_engine::node::{Node, NodeData};
use web_engine::interpreter::Interpreter;

mod tokenizer;
mod html_token;
mod lexer;
mod parse_error;
mod node;
mod comment;
mod character_data;
mod html_document_parser;


fn main() {
    let mut source_html_file_path: String = String::from("");

    let args: Vec<String> = env::args().collect();

        if args.len() == 2 {
            if args[1] == "js" {
                let mut interpreter = Interpreter::new();
                interpreter.run_prompt();
            } else {
                source_html_file_path = args[1].to_string();
                let mut tokenizer = tokenizer::Tokenizer::new(String::from(source_html_file_path));
                tokenizer.start();
            }
        } else if args.len() == 3 {
            if args[1] == "js" {
                let mut interpreter = Interpreter::new();
                interpreter.run_file(args[2].to_string());
            }
        }
}
