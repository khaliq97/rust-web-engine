// This file contains the AST (Abstract Syntax Tree) structures that are shared
// between the Parser and Interpreter.

use crate::token::{Token, Literal};

// https://tc39.es/ecma262/#prod-Statement
pub enum Statement {
    // TODO: Support a list of VariableDeclaration's as seen in the spec
    // Currently we only support one declaration on a single line
    VariableStatement(Box<VariableDeclarationStatement>),
    ExpressionStatement(Box<ExpressionStatement>)
}

// https://tc39.es/ecma262/#prod-VariableStatement
pub struct VariableStatement {
    pub declarations: Vec<VariableDeclarationStatement>,
}

// https://tc39.es/ecma262/#prod-VariableDeclaration
pub struct VariableDeclarationStatement {
    pub binding_identifier: Token,
    //TODO: The initializer should be of type AssignmentExpression(https://tc39.es/ecma262/#prod-AssignmentExpression)
    pub initializer: Option<Box<ExpressionStatement>>
}

// https://tc39.es/ecma262/#prod-AssignmentExpression
pub enum AssignmentExpression {
    LeftHandSideExpression(Box<LeftHandSideExpression>),
}

// https://tc39.es/ecma262/#prod-LeftHandSideExpression
pub struct LeftHandSideExpression {
    // NewExpression TODO
    //  -> MemberExpression TODO
    //      -> PrimaryExpression (TODO: We're representing this as a ExpressionStatement for now, spec is confusing me)
    pub expression: Box<ExpressionStatement>,
}

pub enum ExpressionStatement {
    BinaryExpression(Box<BinaryExpression>),
    LiteralExpression(Box<LiteralExpression>),
    ParenthesizedExpression(Box<ParenthesizedExpression>),
    UnaryExpression(Box<UnaryExpression>),
    VariableExpression(Box<VariableExpression>),
}

pub struct VariableExpression {
    pub binding_identifier: Token,
}

pub struct BinaryExpression {
    pub left: Box<ExpressionStatement>,
    pub right: Box<ExpressionStatement>,
    pub operator: Token,
}

#[derive(Debug)]
pub struct LiteralExpression {
    pub value: Literal,
}

pub struct ParenthesizedExpression {
    pub expression: Box<ExpressionStatement>,
}

pub struct UnaryExpression {
    pub operator: Token,
    pub right: Box<ExpressionStatement>,
}

pub trait Accept<R> {
    fn accept<V: AstVisitor<R>>(&self, visitor: &mut V) -> R;
}

pub trait AstVisitor<R> {
    fn visit_expression_statement(&mut self, expression: &ExpressionStatement) -> R;
    fn visit_binary(&mut self, expression: &BinaryExpression) -> R;
    fn visit_literal(&mut self, expression: &LiteralExpression) -> R;
    fn visit_parenthesized(&mut self, expression: &ParenthesizedExpression) -> R;
    fn visit_unary(&mut self, expression: &UnaryExpression) -> R;
    fn visit_variable_statement(&mut self, expression: &VariableDeclarationStatement) -> R;
    fn visit_variable_expression(&mut self, expression: &VariableExpression) -> R;
}

impl<R> Accept<R> for Statement {
    fn accept<V: AstVisitor<R>>(&self, visitor: &mut V) -> R {
        match self {
            Statement::ExpressionStatement(e) => { visitor.visit_expression_statement(e) }
            Statement::VariableStatement(v) => { visitor.visit_variable_statement(v) }
        }
    }
}

impl<R> Accept<R> for ExpressionStatement {
    fn accept<V: AstVisitor<R>>(&self, visitor: &mut V) -> R {
        match self {
            ExpressionStatement::BinaryExpression(b) => visitor.visit_binary(b),
            ExpressionStatement::LiteralExpression(l) => visitor.visit_literal(l),
            ExpressionStatement::ParenthesizedExpression(p) => visitor.visit_parenthesized(p),
            ExpressionStatement::UnaryExpression(u) => visitor.visit_unary(u),
            ExpressionStatement::VariableExpression(v) => visitor.visit_variable_expression(v),
            _=> unimplemented!()
        }
    }
}
