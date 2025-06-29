// This file contains the ASTPrettyPrinter implementation that was extracted from interpreter.rs

use crate::ast::{AstVisitor, ExpressionStatement, BinaryExpression, LiteralExpression, ParenthesizedExpression, UnaryExpression, IdentifierExpression, VariableDeclarationStatement, Accept, CallExpression, BlockStatement, Statement, ObjectLiteralExpression, AssignmentExpression};
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

    fn parenthesize_statement(&mut self, name: String, exprs: &[Statement]) -> String {
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
            ExpressionStatement::IdentifierExpression(node) => {
                return self.parenthesize(
                    format!("VariableExpression {:?}", node.binding_identifier.lexeme),
                    vec![]
                )
            }
            ExpressionStatement::ParenthesizedExpression(_) => {
                return self.visit_parenthesized(node);
            },
            ExpressionStatement::CallExpression(node) => {
                let mut args_to_string: String = String::new();
                args_to_string.push_str("(");
                for arg in &node.arguments {
                    args_to_string.push_str(arg.accept(self).as_str());
                    args_to_string.push_str(", ");
                }
                args_to_string.push_str(")");
                return self.parenthesize(
                    format!("CallExpression args: {:?}", args_to_string),
                    vec![&*node.callee]
                )
            },
            ExpressionStatement::ObjectLiteralExpression(node) => {
                return self.parenthesize(
                    format!("ObjectLiteralExpression"),
                    vec![]
                )
            },
            ExpressionStatement::AssignmentExpression(node) => {
                return self.parenthesize(
                    format!("AssignmentExpression"),
                    vec![&*node.left_hand_side_expression, &*node.expression]
                )
            }
        }
    }

    fn visit_identifier_expression(&mut self, expression: &IdentifierExpression) -> String {
        return self.parenthesize(
            format!("IdentifierExpression {:?}", expression.binding_identifier.lexeme),
            vec![]
        )
    }

    fn visit_unary(&mut self, node: &UnaryExpression) -> String {
        self.parenthesize(
            format!("UnaryExpression {:?}", node.operator.token_type),
            vec![&*node.right]
        )
    }

    fn visit_assignment_expression(&mut self, expression: &AssignmentExpression) -> String {
        return self.parenthesize(
            format!("AssignmentExpression"),
            vec![&*expression.left_hand_side_expression, &*expression.expression]
        )
    }

    fn visit_variable_declaration(&mut self, expression: &VariableDeclarationStatement) -> String {
        match &expression.initializer {
            Some(ref init_node) => {
                let assignment_expression_string = self.visit_assignment_expression(init_node);
                return self.parenthesize(
                    format!("[VariableDeclarationStatement] BindingIdentifier:{:?}, Initializer:{:?}", expression.binding_identifier.lexeme,
                            assignment_expression_string),
                    vec![]
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

    fn visit_object_literal_expression(&mut self, expression: &ObjectLiteralExpression) -> String {
        let mut assignment_expressions: Vec<String> = Vec::new();
        for property_definition in &expression.property_definitions {
            assignment_expressions.push(self.visit_assignment_expression(&property_definition.assignment_expression));
        }
        return self.parenthesize(
            format!("ObjectLiteralExpression PropertyDefinitions {}", assignment_expressions.join(", ")),
            vec![]
        )
    }

    fn visit_call_expression(&mut self, expression: &CallExpression) -> String {
        let mut args_to_string: String = String::new();
        args_to_string.push_str("(");
        for arg in &expression.arguments {
            args_to_string.push_str(arg.accept(self).as_str());
            args_to_string.push_str(", ");
        }
        args_to_string.push_str(")");
        return self.parenthesize(
            format!("CallExpression args: {:?}", args_to_string),
            vec![&*expression.callee]
        )
    }

    fn visit_block_statement(&mut self, expression: &BlockStatement) -> String {
        self.parenthesize_statement(
            format!("BlockStatement"),
            &*expression.statements
        )
    }
}
