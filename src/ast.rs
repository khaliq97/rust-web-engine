// This file contains the AST (Abstract Syntax Tree) structures that are shared
// between the Parser and Interpreter.

use std::rc::Rc;
use crate::token::{Token, Literal};

// https://tc39.es/ecma262/#prod-Statement
pub enum Statement {
    // TODO: Support a list of VariableDeclaration's as seen in the spec
    // Currently we only support one declaration on a single line
    VariableStatement(Box<VariableDeclarationStatement>),
    ExpressionStatement(Box<ExpressionStatement>),
    BlockStatement(Box<BlockStatement>)
}

#[derive(Debug)]
// https://tc39.es/ecma262/#prod-PropertyDefinition
pub struct PropertyDefinition {
    pub(crate) property_name: PropertyName,
    pub(crate) assignment_expression: AssignmentExpression,
}


#[derive(Debug)]
//https://tc39.es/ecma262/#prod-PropertyName
// TODO: Support computed property names: https://tc39.es/ecma262/#prod-ComputedPropertyName
pub enum PropertyName {
    IdentifierName(Token),
    LiteralPropertyName(Literal),
}

#[derive(Debug)]
// https://tc39.es/ecma262/#prod-ObjectLiteral
pub struct ObjectLiteralExpression {
    pub property_definitions: Vec<PropertyDefinition>,
}

// https://tc39.es/ecma262/#prod-VariableStatement
pub struct VariableStatement {
    pub declarations: Vec<VariableDeclarationStatement>,
}

// https://tc39.es/ecma262/#prod-VariableDeclaration
pub struct VariableDeclarationStatement {
    pub binding_identifier: Token,
    //TODO: The initializer should be of type AssignmentExpression(https://tc39.es/ecma262/#prod-AssignmentExpression)
    pub initializer: Option<Box<AssignmentExpression>>
}

#[derive(Debug)]
// https://tc39.es/ecma262/#prod-AssignmentExpression
pub struct AssignmentExpression {
    // https://tc39.es/ecma262/#prod-LeftHandSideExpression
    // NewExpression TODO
    //  -> MemberExpression TODO
    //      -> PrimaryExpression (TODO: We're representing this as a ExpressionStatement for now, spec is confusing me)
    // At some point we'll split the LeftHandSideExpression out to it's own struct but this is ok for now
    pub expression: Rc<ExpressionStatement>,
    pub left_hand_side_expression: Rc<ExpressionStatement>
}

// https://tc39.es/ecma262/#prod-FunctionBody
pub struct FunctionBody {
    // https://tc39.es/ecma262/#prod-FunctionStatementList
    // -> https://tc39.es/ecma262/#prod-StatementList
    //  -> https://tc39.es/ecma262/#prod-StatementListItem
    //   -> https://tc39.es/ecma262/#prod-Statement
    statements: Vec<Statement>,

}

// https://tc39.es/ecma262/#prod-FormalParameter
pub struct FormalParameter {
    // https://tc39.es/ecma262/#prod-BindingElement
    // -> https://tc39.es/ecma262/#prod-SingleNameBinding
    //  -> https://tc39.es/ecma262/#prod-BindingIdentifier
    binding_identifier: Token,

}
// https://tc39.es/ecma262/#prod-FormalParameters
pub struct FormalParameters {
    parameters: Vec<FormalParameter>,
}

//https://tc39.es/ecma262/#prod-FunctionDeclaration
pub struct FunctionDeclaration {
    pub binding_identifier: Token,
    pub formal_parameters: FormalParameters,
    pub function_body: FunctionBody,
}

#[derive(Debug)]
// https://tc39.es/ecma262/#prod-CallExpression
pub struct CallExpression {
    pub(crate) callee: Box<ExpressionStatement>,
    pub(crate) paren: Token,
    pub(crate) arguments: Vec<ExpressionStatement>,
}


// https://tc39.es/ecma262/#prod-BlockStatement
// BlockStatement[Yield, Await, Return] :
//  Block[?Yield, ?Await, ?Return]
pub struct BlockStatement {
    //  Block[?Yield, ?Await, ?Return]
    //  ->  { StatementList[?Yield, ?Await, ?Return]opt }
    //    ->  StatementListItem[Yield, Await, Return] :
    //          Statement[?Yield, ?Await, ?Return]
    //          Declaration[?Yield, ?Await]
    pub statements: Vec<Statement>,
}

pub trait Callable {
    fn call() { 
        
    }
}

#[derive(Debug)]
pub enum ExpressionStatement {
    BinaryExpression(Box<BinaryExpression>),
    LiteralExpression(Box<LiteralExpression>),
    ParenthesizedExpression(Box<ParenthesizedExpression>),
    UnaryExpression(Box<UnaryExpression>),
    IdentifierExpression(Box<IdentifierExpression>),
    CallExpression(Box<CallExpression>),
    ObjectLiteralExpression(Box<ObjectLiteralExpression>),
    AssignmentExpression(Box<AssignmentExpression>)
}

#[derive(Debug)]
pub struct IdentifierExpression {
    pub binding_identifier: Token,
}

#[derive(Debug)]
pub struct BinaryExpression {
    pub left: Box<ExpressionStatement>,
    pub right: Box<ExpressionStatement>,
    pub operator: Token,
}

#[derive(Debug)]
pub struct LiteralExpression {
    pub value: Literal,
}

#[derive(Debug)]
pub struct ParenthesizedExpression {
    pub expression: Box<ExpressionStatement>,
}

#[derive(Debug)]
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
    fn visit_variable_declaration(&mut self, expression: &VariableDeclarationStatement) -> R;
    fn visit_identifier_expression(&mut self, expression: &IdentifierExpression) -> R;
    fn visit_call_expression(&mut self, expression: &CallExpression) -> R;
    fn visit_block_statement(&mut self, expression: &BlockStatement) -> R;
    fn visit_object_literal_expression(&mut self, expression: &ObjectLiteralExpression) -> R;
    fn visit_assignment_expression(&mut self, expression: &AssignmentExpression) -> R;
}

impl<R> Accept<R> for Statement {
    fn accept<V: AstVisitor<R>>(&self, visitor: &mut V) -> R {
        match self {
            Statement::ExpressionStatement(e) => { visitor.visit_expression_statement(e) }
            Statement::VariableStatement(v) => { visitor.visit_variable_declaration(v) }
            Statement::BlockStatement(b) => { visitor.visit_block_statement(b) }
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
            ExpressionStatement::IdentifierExpression(v) => visitor.visit_identifier_expression(v),
            ExpressionStatement::CallExpression(c) => visitor.visit_call_expression(c),
            ExpressionStatement::ObjectLiteralExpression(o) => visitor.visit_object_literal_expression(o),
            ExpressionStatement::AssignmentExpression(a) => visitor.visit_assignment_expression(a),
            _=> unimplemented!()
        }
    }
}
