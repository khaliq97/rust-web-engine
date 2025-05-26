// This file contains the ASTPrettyPrinter implementation that was extracted from interpreter.rs

use crate::ast::{
    AstVisitor, ExpressionStatement, BinaryExpression, LiteralExpression,
    ParenthesizedExpression, UnaryExpression, VariableExpression, VariableDeclarationStatement,
    Accept
};
use crate::token::Literal;

pub struct ASTPrettyPrinter;

impl ASTPrettyPrinter {
    fn parenthesize(&mut self, name: String, exprs: Vec<&ExpressionStatement>) -> String {
        let mut builder = String::new();

        builder.push('(');
        builder.push_str(&name);

        for expr in exprs {
            builder.push(' ');
            builder.push_str(&expr.accept(self));
        }

        builder.push(')');

        builder
    }
}

impl AstVisitor<String> for ASTPrettyPrinter {
    fn visit_expression_statement(&mut self, expression: &ExpressionStatement) -> String {
        return expression.accept(self);
    }

    fn visit_binary(&mut self, node: &BinaryExpression) -> String {
        self.parenthesize(
            format!("BinaryExpression {:?}", node.operator.token_type),
            vec![&*node.left, &*node.right]
        )
    }

    fn visit_literal(&mut self, node: &LiteralExpression) -> String {
        match &node.value {
            Literal::Numeric(n) => format!("NumericLiteral {}", n.to_string()),
            Literal::String(s) => format!("StringLiteral {}", s),
            &Literal::Boolean(b) => format!("BooleanLiteral {}", b),
            &Literal::Null() => "NullLiteral null".to_string()
        }
    }

    fn visit_parenthesized(&mut self, node: &ParenthesizedExpression) -> String {
        match &*node.expression {
            ExpressionStatement::BinaryExpression(node) => {
                return self.parenthesize(
                    format!("ParenthesizedExpression {:?}", node.operator.token_type),
                    vec![&*node.left, &*node.right]
                )
            },
            ExpressionStatement::LiteralExpression(node) => {
                return self.parenthesize(
                    format!("ParenthesizedExpression {:?}", node.value),
                    vec![]
                )
            },
            ExpressionStatement::UnaryExpression(node) => {
                return self.parenthesize(
                    format!("ParenthesizedExpression {:?}", node.operator.token_type),
                    vec![&*node.right]
                )
            },
            ExpressionStatement::VariableExpression(node) => {
                return self.parenthesize(
                    format!("VariableExpression {:?}", node.binding_identifier.literal),
                    vec![]
                )
            }
            ExpressionStatement::ParenthesizedExpression(_) => {
                return self.visit_parenthesized(node);
            }
        }
    }

    fn visit_variable_expression(&mut self, expression: &VariableExpression) -> String {
        return self.parenthesize(
            format!("VariableExpression {:?}", expression.binding_identifier.literal),
            vec![]
        )
    }

    fn visit_unary(&mut self, node: &UnaryExpression) -> String {
        self.parenthesize(
            format!("UnaryExpression {:?}", node.operator.token_type),
            vec![&*node.right]
        )
    }

    fn visit_variable_statement(&mut self, expression: &VariableDeclarationStatement) -> String {
        match &expression.initializer {
            Some(ref init_node) => {
                return self.parenthesize(
                    format!("VariableDeclarationStatement {:?}", expression.binding_identifier.lexeme),
                    vec![&**init_node]
                )
            },
            None => {
                return self.parenthesize(
                    format!("VariableDeclarationStatement {:?}", expression.binding_identifier.lexeme),
                    vec![]
                )
            }
        }
    }
}
