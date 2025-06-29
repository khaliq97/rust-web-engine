// This file contains the Parser implementation that was extracted from interpreter.rs

use std::rc::Rc;
use serde_json::de::Read;
use crate::token::{Token, TokenType, Literal};
use crate::ast::{Statement, VariableDeclarationStatement, ExpressionStatement, BinaryExpression, LiteralExpression, ParenthesizedExpression, UnaryExpression, IdentifierExpression, CallExpression, BlockStatement, ObjectLiteralExpression, AssignmentExpression, PropertyDefinition, PropertyName};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser { tokens, current: 0 }
    }

    pub fn expression(&mut self) -> ExpressionStatement {
        return self.assignment_expression();
    }

    fn assignment_expression(&mut self) -> ExpressionStatement {
        let expression = self.equality();

        if self.match_token(vec![TokenType::EQUAL]) {
            let equals = &self.previous();

            match expression {
                ExpressionStatement::IdentifierExpression(var_expr) => {
                    return ExpressionStatement::AssignmentExpression(Box::new(AssignmentExpression {
                        left_hand_side_expression: Rc::new(ExpressionStatement::IdentifierExpression(Box::new(*var_expr))),
                        expression: Rc::new(self.assignment_expression())
                    }))
                },
                _ => {
                    println!("{:?}: Invalid assignment target.", equals);
                }
            }
        }

        return expression;
    }

    pub fn statement(&mut self) -> Statement {
        // https://tc39.es/ecma262/#sec-asi-interesting-cases-in-statement-lists
        // TODO: Handle automatic semi colon insertion, see spec:
        if self.peek().token_type == TokenType::SEMICOLON {
            self.advance();
        } else if self.match_token(vec![TokenType::LEFT_BRACE]) {
            return self.block_statement();
        }
        return self.expression_statement()
    }

    pub fn block_statement(&mut self) -> Statement {
        let mut statements: Vec<Statement> = Vec::new();
        while !self.check(TokenType::RIGHT_BRACE) && !self.is_at_end() {
            statements.push(self.declaration());
        }

        if self.peek().token_type == TokenType::SEMICOLON {
            self.advance();
        }

        self.consume(TokenType::RIGHT_BRACE, "Expect '}' after block.".to_string());
        return Statement::BlockStatement(Box::new(BlockStatement { statements }))
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
        let mut initializer: Option<Box<AssignmentExpression>> = None;

        if self.match_token(vec![TokenType::EQUAL]) {
            initializer = Some(Box::new(AssignmentExpression {
                left_hand_side_expression: Rc::new(ExpressionStatement::IdentifierExpression(Box::new(IdentifierExpression { binding_identifier: name.clone() }))),
                expression: Rc::new(self.expression()),
            }));

            return Statement::VariableStatement(Box::new(VariableDeclarationStatement {
                binding_identifier: name,
                initializer }))
        }

        return Statement::VariableStatement(Box::new(VariableDeclarationStatement { binding_identifier: name, initializer }))
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

        return self.call_expression()
    }

    fn call_expression(&mut self) -> ExpressionStatement {
        let mut expression: ExpressionStatement = self.primary();
        loop {
            if self.match_token(vec![TokenType::LeftParen]) {
                expression = self.finish_call(expression);
            } else {
                break;
            }
        }

        return expression;
    }

    fn finish_call(&mut self, callee: ExpressionStatement) -> ExpressionStatement {
        let mut arguments: Vec<ExpressionStatement> = Vec::new();
            arguments.push(self.expression());

            while self.match_token(vec![TokenType::COMMA]) {
                arguments.push(self.expression());
                if self.check(TokenType::RIGHT_PAREN) {
                    break;
                }
            }

        let paren = self.consume(TokenType::RIGHT_PAREN, "Expect ')' after arguments.".to_string());

        return ExpressionStatement::CallExpression(Box::new(CallExpression { callee: Box::new(callee), paren: paren.clone(), arguments }))
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
            return ExpressionStatement::IdentifierExpression(Box::new(IdentifierExpression { binding_identifier: self.previous().clone() }))
        }

        if self.match_token(vec![TokenType::LEFT_BRACE]) {
            // https://tc39.es/ecma262/#sec-static-semantics-propertynamelist
            let mut property_name_list: Vec<PropertyDefinition> = Vec::new();

            match self.create_property_definition() {
                Some(property_name) => {
                    property_name_list.push(property_name);

                    while self.match_token(vec![TokenType::COMMA]) {
                        property_name_list.push(self.create_property_definition().unwrap());
                        if self.check(TokenType::RIGHT_BRACE) {
                            break;
                        }
                    }
                    self.consume(TokenType::RIGHT_BRACE, "Expect '}' after expression.".to_string());

                },
                None => {
                    self.consume(TokenType::RIGHT_BRACE, "Expect '}' after expression.".to_string());
                }
            }

            return ExpressionStatement::ObjectLiteralExpression(Box::new(ObjectLiteralExpression { property_definitions: property_name_list }))
        }

        if self.match_token(vec![TokenType::LeftParen]) {
            let expression = self.expression();
            self.consume(TokenType::RIGHT_PAREN, "Expect ')' after expression.".to_string());
            return ExpressionStatement::ParenthesizedExpression(Box::new(ParenthesizedExpression { expression: Box::new(expression) }))
        }

        // Default case - maybe should return an option
        ExpressionStatement::LiteralExpression(Box::new(LiteralExpression { value: Literal::Null() }))
    }


    // https://tc39.es/ecma262/#sec-static-semantics-propertynamelist
    fn create_property_definition(&mut self) -> Option<PropertyDefinition> {
        if self.match_token(vec![TokenType::IDENTIFIER, TokenType::NUMBER, TokenType::STRING]) {
            // 1. Let propName be the PropName of PropertyDefinition.

            // TODO: Implement proper getting of PropName https://tc39.es/ecma262/#sec-static-semantics-propname
            let prop_name = self.previous().clone();

            self.consume(TokenType::COLON, "Uncaught SyntaxError: missing : after property id".to_string());

            if prop_name.token_type == TokenType::IDENTIFIER {
                let expression = self.expression();

                // 3. Return « propName ».
                return Some(PropertyDefinition { property_name: PropertyName::IdentifierName(prop_name.clone()),
                    assignment_expression: AssignmentExpression { left_hand_side_expression: Rc::new(ExpressionStatement::IdentifierExpression(Box::new(IdentifierExpression { binding_identifier: prop_name }))), expression: Rc::new(expression) }});
            } else {
                match prop_name.literal {
                    Some(Literal::String(ref value)) => {
                        let expression = self.expression();
                        // 3. Return « propName ».
                        return Some(PropertyDefinition { property_name:  PropertyName::LiteralPropertyName(Literal::String(value.clone())),
                            assignment_expression: AssignmentExpression { left_hand_side_expression: Rc::new(ExpressionStatement::IdentifierExpression(Box::new(IdentifierExpression { binding_identifier: prop_name }))), expression: Rc::new(expression) }});

                    },
                    _ => { unimplemented!() }
                }
            }

            // https://tc39.es/ecma262/#sec-object-initializer-static-semantics-early-errors
        }

        // 2. If propName is empty, return a new empty List.
        return None;
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
