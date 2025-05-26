// This file contains the Parser implementation that was extracted from interpreter.rs

use crate::token::{Token, TokenType, Literal};
use crate::ast::{
    Statement, VariableDeclarationStatement, ExpressionStatement,
    BinaryExpression, LiteralExpression, ParenthesizedExpression, UnaryExpression, VariableExpression
};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser { tokens, current: 0 }
    }

    pub fn expression(&mut self) -> ExpressionStatement {
        return self.equality();
    }

    pub fn statement(&mut self) -> Statement {
        // Add support for statements here...

        // https://tc39.es/ecma262/#sec-asi-interesting-cases-in-statement-lists
        // TODO: Handle automatic semi colon insertion, see spec:

        return self.expression_statement()
    }

    pub fn declaration(&mut self) -> Statement {
        // https://tc39.es/ecma262/#prod-VariableStatement
        if self.match_token(vec![TokenType::VAR]) {
            return self.var_declaration();
        }

        return self.statement();

        // TODO: Error handling
    }

    fn var_declaration(&mut self) -> Statement {
        let name = self.consume(TokenType::IDENTIFIER, "missing variable name".to_string()).clone();
        let mut initializer: Option<Box<ExpressionStatement>> = None;

        if self.match_token(vec![TokenType::EQUAL]) {
            initializer = Option::Some(Box::new(self.expression()));
            return Statement::VariableStatement(Box::new(VariableDeclarationStatement { binding_identifier: name, initializer: initializer }))
        }

        return Statement::VariableStatement(Box::new(VariableDeclarationStatement { binding_identifier: name, initializer: initializer }))
    }


    fn expression_statement(&mut self) -> Statement {
        let expression = self.expression();
        return Statement::ExpressionStatement(Box::new(expression));
    }

    fn equality(&mut self) -> ExpressionStatement {
        let mut expression: ExpressionStatement = self.comparison();

        while(self.match_token(vec![TokenType::BANG_EQUAL, TokenType::EQUAL_EQUAL])) {
            let operator = self.previous().clone();
            let right = self.comparison();
            expression = ExpressionStatement::BinaryExpression(Box::new(BinaryExpression { left: Box::new(expression), right: Box::new(right), operator }));
        }
        return expression;
    }

    fn comparison(&mut self) -> ExpressionStatement {
        let mut expression: ExpressionStatement = self.term();

        while self.match_token(vec![TokenType::GREATER, TokenType::GREATER_EQUAL, TokenType::LESS, TokenType::LESS_EQUAL]) {
            let operator = self.previous().clone();
            let right = self.term();
            expression = ExpressionStatement::BinaryExpression(Box::new(BinaryExpression { left: Box::new(expression), right: Box::new(right), operator }));
        }

        return expression;
    }

    fn term(&mut self) -> ExpressionStatement {
        let mut expression: ExpressionStatement = self.factor();

        while self.match_token(vec![TokenType::MINUS, TokenType::PLUS]) {
            let operator = self.previous().clone();
            let right = self.factor();
            expression = ExpressionStatement::BinaryExpression(Box::new(BinaryExpression { left: Box::new(expression), right: Box::new(right), operator }));
        }

        return expression;
    }

    fn factor(&mut self) -> ExpressionStatement {
        let mut expression: ExpressionStatement = self.unary();

        while self.match_token(vec![TokenType::SLASH, TokenType::STAR]) {
            let operator = self.previous().clone();
            let right = self.unary();
            expression = ExpressionStatement::BinaryExpression(Box::new(BinaryExpression { left: Box::new(expression), right: Box::new(right), operator }));
        }

        return expression;
    }

    fn unary(&mut self) -> ExpressionStatement {
        if self.match_token(vec![TokenType::BANG, TokenType::MINUS, TokenType::PLUS]) {
            let operator = self.previous().clone();
            let right = self.unary();
            return ExpressionStatement::UnaryExpression(Box::new(UnaryExpression { operator, right: Box::new(right) }))
        }

        return self.primary()
    }

    fn primary(&mut self) -> ExpressionStatement {
        if self.match_token(vec![TokenType::FALSE]) {
            return  ExpressionStatement::LiteralExpression(Box::new(LiteralExpression { value: Literal::Boolean(false) }));
        }

        if self.match_token(vec![TokenType::TRUE]) {
            return ExpressionStatement::LiteralExpression(Box::new(LiteralExpression { value: Literal::Boolean(true) }))
        }

        if self.match_token(vec![TokenType::NULL]) {
            return ExpressionStatement::LiteralExpression(Box::new(LiteralExpression { value: Literal::Null() }))
        }

        if self.match_token(vec![TokenType::NUMBER, TokenType::STRING]) {
            let literal_value = self.previous().literal.clone().unwrap();
            return ExpressionStatement::LiteralExpression(Box::new(LiteralExpression { value: literal_value }))
        }

        // https://tc39.es/ecma262/#prod-VariableDeclaration
        if self.match_token(vec![TokenType::IDENTIFIER]) {
            return ExpressionStatement::VariableExpression(Box::new(VariableExpression { binding_identifier: self.previous().clone() }))
        }

        if self.match_token(vec![TokenType::LeftParen]) {
            let expression = self.expression();
            self.consume(TokenType::RIGHT_PAREN, "Expect ')' after expression.".to_string());
            return ExpressionStatement::ParenthesizedExpression(Box::new(ParenthesizedExpression { expression: Box::new(expression) }))
        }

        // Default case - maybe should return an option
        ExpressionStatement::LiteralExpression(Box::new(LiteralExpression { value: Literal::Null() }))
    }

    fn consume(&mut self, token_type: TokenType, message: String) -> &Token {
        if self.check(token_type.clone()) {
            let token = self.advance();
            return token;
        }

        if (token_type == TokenType::EOF) {
            println!("Uncaught SyntaxError: {} at end", message);
            return self.peek();
        } else {
            println!("Uncaught SyntaxError: {} at line {}", message, self.peek().line);
            return self.peek();
        }
    }

    fn match_token(&mut self, tokens: Vec<TokenType>) -> bool {
        for token in tokens {
            if self.check(token) {
                self.advance();
                return true;
            }
        }

        return false;
    }

    fn check(&self, token: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        return self.peek().token_type == token;
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current = self.current + 1;
        }

        return self.previous();
    }

    fn is_at_end(&self) -> bool {
        match self.peek() {
            Token { token_type: TokenType::EOF, .. } => true,
            _ => false,
        }
    }

    fn peek(&self) -> &Token {
        return &self.tokens[self.current];
    }

    fn previous(&self) -> &Token {
        return &self.tokens[self.current - 1];
    }

    pub fn parse(&mut self) -> Vec<Statement> {
        let mut statements: Vec<Statement> = Vec::new();
        while !self.is_at_end() {
            statements.push(self.declaration());
        }

        return statements;
    }
}
