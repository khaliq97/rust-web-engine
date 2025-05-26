use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::panic::catch_unwind;
use log::error;
use crate::token::{Token, TokenType, Literal};
use crate::scanner::Scanner;
use crate::parser::Parser;
use crate::ast::{
    Statement, ExpressionStatement, BinaryExpression, LiteralExpression,
    ParenthesizedExpression, UnaryExpression, VariableExpression, VariableDeclarationStatement,
    AstVisitor, Accept
};
use crate::ast_printer::ASTPrettyPrinter;

pub struct Interpreter {
    had_error: bool
}

// https://tc39.es/ecma262/#sec-ecmascript-language-types-symbol-type
#[derive(Debug)]
struct JSSymbol {
    description: String,
}

// https://tc39.es/ecma262/#sec-object-type
#[derive(Debug)]
struct JSObject {

}

// https://tc39.es/ecma262/#sec-ecmascript-language-types-number-type
// TODO: Support BigInt https://tc39.es/ecma262/#sec-ecmascript-language-types-bigint-type
type Number = f64;

// https://tc39.es/ecma262/#sec-ecmascript-language-types
#[derive(Debug)]
enum JSValue {
    Undefined,
    Boolean(bool),
    String(String),
    Symbol(JSSymbol),
    Numeric(Number),
    Object(JSObject),
    Null
}

impl AstVisitor<JSValue> for Interpreter {
    fn visit_expression_statement(&mut self, expression: &ExpressionStatement) -> JSValue {
        return self.evaluate(expression);
    }

    // https://tc39.es/ecma262/#sec-evaluatestringornumericbinaryexpression
    fn visit_binary(&mut self, expression: &BinaryExpression) -> JSValue {
        // 1. Let lRef be ? Evaluation of leftOperand.
        let left_expression = self.evaluate(&*expression.left);

        // 2. Let lVal be ? GetValue(lRef).
        let left_value = Interpreter::get_value(left_expression);

        // 3. Let rRef be ? Evaluation of rightOperand.
        let right_expression = self.evaluate(&*expression.right);

        // 4. Let rVal be ? GetValue(rRef).
        let right_value = Interpreter::get_value(right_expression);

        let value = Interpreter::apply_string_or_numeric_binary_operator(left_value, right_value, &expression.operator.token_type);
        return value;
    }

    fn visit_literal(&mut self, expression: &LiteralExpression) -> JSValue {
        match &expression.value {
            Literal::String(value) => {
                return JSValue::String(value.to_string());
            }
            Literal::Numeric(value) => {
                return JSValue::Numeric(*value as Number);
            }
            Literal::Boolean(value) => {
                return JSValue::Boolean(*value);
            }
            Literal::Null() => {
                return JSValue::Null;
            }
        }
    }

    fn visit_parenthesized(&mut self, expression: &ParenthesizedExpression) -> JSValue {
        return self.evaluate(&*expression.expression);
    }


    // https://tc39.es/ecma262/#prod-UnaryExpression
    fn visit_unary(&mut self, expression: &UnaryExpression) -> JSValue {
        // 1. Let expr be ? Evaluation of UnaryExpression.
        let right = self.evaluate(&expression.right);

        match expression.operator.token_type {
            // https://tc39.es/ecma262/#sec-unary-plus-operator-runtime-semantics-evaluation
            TokenType::PLUS => {
                // 2. Return ? ToNumber(? GetValue(expr)).
                return Interpreter::to_number(right);
            },
            // https://tc39.es/ecma262/#sec-unary-minus-operator-runtime-semantics-evaluation
            TokenType::MINUS => {
                // 2. Let oldValue be ? ToNumeric(? GetValue(expr)).
                // TODO: Implement GetValue(expr)
                let old_value = Interpreter::to_numeric(right);

                // 3. If oldValue is a Number, then
                match old_value {
                    JSValue::Numeric(value) => {
                        //a. TODO: Return Number::unaryMinus(oldValue).
                        // https://tc39.es/ecma262/#sec-numeric-types-number-unaryMinus
                        // Currently we just return the negative value and don't check for NaN.
                        return JSValue::Numeric(-value);
                    },
                    // 4. Else
                    _ => {
                        // a. Assert: oldValue is a BigInt.
                        // b. Return BigInt::unaryMinus(oldValue).
                        todo!()
                    }
                }
            },
            // https://tc39.es/ecma262/#sec-bitwise-not-operator-runtime-semantics-evaluation
            TokenType::BITWISE_NOT => {
                todo!();
            },
            // https://tc39.es/ecma262/#sec-logical-not-operator-runtime-semantics-evaluation
            TokenType::BANG => {
                // 2. Let oldValue be ToBoolean(? GetValue(expr)).
                let old_value = Interpreter::to_boolean(right);

                match old_value {
                    // 3. If oldValue is true, return false.
                    JSValue::Boolean(true) => {
                        return JSValue::Boolean(false);
                    },
                    // 4. Return true.
                    _ => {
                        return JSValue::Boolean(true);
                    }
                }
            }
            _ => { todo!() }
        }

        // https://tc39.es/ecma262/#sec-unary-plus-operator-runtime-semantics-evaluation

        // https://tc39.es/ecma262/#sec-unary-minus-operator-runtime-semantics-evaluation

        // TODO: https://tc39.es/ecma262/#sec-bitwise-not-operator

        // https://tc39.es/ecma262/#sec-logical-not-operator


    }

    fn visit_variable_statement(&mut self, expression: &VariableDeclarationStatement) -> JSValue {
        match &expression.initializer {
            Some(init) => {
                return JSValue::Undefined;
            },
            None => {
                return JSValue::Undefined;
            }
        }
    }

    fn visit_variable_expression(&mut self, expression: &VariableExpression) -> JSValue {
        return JSValue::Undefined;
    }
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter { had_error: false }
    }

    pub fn run_file(&mut self, path: String) {
        let file = File::open(path).expect("File could not opened!");
        let mut reader = BufReader::new(file);
        let mut source = String::new();
        reader.read_to_string(&mut source).expect("File could not be read!");
        self.run(source);

        if self.had_error {
            std::process::exit(65);
        }
    }

    pub fn run_prompt(&mut self) {
        loop {
            print!("> ");
            std::io::stdout().flush().unwrap();
            let mut line = String::new();
            std::io::stdin().read_line(&mut line).expect("Failed to read line");
            self.run(line);
            self.had_error = false;
        }
    }

    fn run(&mut self, source: String) {
        let mut scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens().clone();

        for token in tokens.iter() {
            println!("{}", token.to_string());
        }

        let mut parser = Parser::new(tokens);
        let statements = parser.parse();
        self.interpret(statements);


    }

    fn error(line: usize, message: String) {
        println!("Error on line {}: {}", line, message);
    }

    fn report(&mut self, line: i64, location: String, message: String) {
        println!("[line {}] Error {}: {}", line, location, message);
        self.had_error = true;
    }

    fn execute(&mut self, statement: &Statement) -> JSValue {
        statement.accept(self)
    }

    fn evaluate(&mut self, expression_statement: &ExpressionStatement) -> JSValue {
        expression_statement.accept(self)
    }

    fn interpret(&mut self, statements: Vec<Statement>)  {
        for statement in statements.iter() {
            let result = self.execute(statement);
            let mut pretty_printer = ASTPrettyPrinter;
            let expression_ast = statement.accept(&mut pretty_printer);
            println!("Parsed expression {}", expression_ast);
            println!("{:?}", result)
        }
    }

    // https://tc39.es/ecma262/#sec-tonumber
    // TODO: Return a normal completion or throw a completion
    fn to_number(value: JSValue) -> JSValue {
        match value {
            // 1. If argument is a Number, return argument.
            JSValue::Numeric(value) => {
                return JSValue::Numeric(value);
            },
            // 2. If argument is either a Symbol or a BigInt, throw a TypeError exception.
            JSValue::Symbol(value) => {
                todo!()
            },
            // 3. If argument is undefined, return NaN.
            // TODO: Support undefined as a global object
            JSValue::Undefined => {
                // TODO: Implement NaN as a global object and not a string
                // https://tc39.es/ecma262/#sec-value-properties-of-the-global-object-nan
                return JSValue::String("NaN".to_string());
            },
            // 4. If argument is either null or false, return +0𝔽.
            JSValue::Null | JSValue::Boolean(false) => {
                return JSValue::Numeric(0.0);
            },
            // 5. If argument is true, return 1𝔽.
            JSValue::Boolean(true) => {
                return JSValue::Numeric(1.0);
            }
            //6. If argument is a String, return StringToNumber(argument).
            JSValue::String(value) => {
                todo!();
            }
            // 7. Assert: argument is an Object.
            JSValue::Object(value) => {
                // 8. Let primValue be ? ToPrimitive(argument, number).
                // 9. Assert: primValue is not an Object.
                // 10. Return ? ToNumber(primValue).
                todo!()
            }

        }
    }


    // https://tc39.es/ecma262/#sec-toprimitive
    fn to_primitive(value: JSValue, preferred_type: Option<JSValue>) -> JSValue {
        match value {
            // 1. If input is an Object, then
            JSValue::Object(value) => {
                todo!();
            },
            _ => {
                return value;
            }
        }
    }

    // https://tc39.es/ecma262/#sec-tonumeric
    fn to_numeric(value: JSValue) -> JSValue {
        // 1. Let primValue be ? ToPrimitive(value, number).
        let prim_value = Interpreter::to_primitive(value, None);

        //2. TODO: If primValue is a BigInt, return primValue.

        //3. Return ? ToNumber(primValue).
        return Interpreter::to_number(prim_value);
    }

    // https://tc39.es/ecma262/#sec-toboolean
    fn to_boolean(value: JSValue) -> JSValue {
        match value {
            //1. If argument is a Boolean, return argument.
            JSValue::Boolean(value) => {
                return JSValue::Boolean(value);
            },
            // 2. If argument is one of undefined, null, +0𝔽, -0𝔽, NaN, 0ℤ, or the empty String, return false. TODO: NaN and 0ℤ
            JSValue::Undefined | JSValue::Null | JSValue::Numeric(0.0) | JSValue::Numeric(-0.0) => {
                return JSValue::Boolean(false);
            },
            JSValue::String(ref s) if s.is_empty() => {
                return JSValue::Boolean(false);
            },
            // 3. If argument is an Object and argument has an [[IsHTMLDDA]] internal slot, return false.
            JSValue::Object(value) => {
                todo!();
            }
            // Handle other cases
            _ => {
                return JSValue::Boolean(true);
            }
        }
    }

    // https://tc39.es/ecma262/#sec-getvalue
    fn get_value(value: JSValue) -> JSValue {
        //1. If V is not a Reference Record, return V.
        return value;

        //TODO: Remaining 3 steps, see spec
    }

    // https://tc39.es/ecma262/#sec-applystringornumericbinaryoperator
    fn apply_string_or_numeric_binary_operator(left: JSValue, right: JSValue, operator: &TokenType) -> JSValue {
        // 1. If opText is +, then
        if operator == &TokenType::PLUS {
            // a. Let lPrim be ? ToPrimitive(lVal).
            let left_primitive = Interpreter::to_primitive(left, None);

            // b. Let rPrim be ? ToPrimitive(rVal).
            let right_primitive = Interpreter::to_primitive(right, None);

            match left_primitive {
                // c. If lPrim is a String or rPrim is a String, then
                JSValue::String(ref value) => {
                    // i. Let lStr be ? ToString(lPrim).
                    let left_string = Interpreter::to_string(left_primitive);

                    // ii. Let rStr be ? ToString(rPrim).
                    let right_string = Interpreter::to_string(right_primitive);

                    match left_string {
                        JSValue::String(ref left_string) => {
                            match right_string {
                                JSValue::String(ref right_string) => {
                                    // iii. Return the string-concatenation of lStr and rStr.
                                    return JSValue::String(format!("{}{}", left_string, right_string));
                                },
                                _ => { panic!("Unexpected right JS value: {:?}", right_string) }
                            }
                        },
                        _ => { panic!("Unexpected left JS value: {:?}", right_string) }
                    }
                },
                _ => {
                    match right_primitive {
                        // c. If lPrim is a String or rPrim is a String, then
                        JSValue::String(ref value) => {
                            let left_string = Interpreter::to_string(left_primitive);
                            let right_string = Interpreter::to_string(right_primitive);

                            match left_string {
                                JSValue::String(ref left_string) => {
                                    match right_string {
                                        JSValue::String(ref right_string) => {
                                            return JSValue::String(format!("{}{}", left_string, right_string));
                                        },
                                        _ => { panic!("Unexpected right JS value: {:?}", right_string) }
                                    }
                                },
                                _ => { panic!("Unexpected left JS value: {:?}", right_string) }
                            }
                        },
                        _ => {
                            // We know the opText is still '+' so apply the addition operation.
                            // https://tc39.es/ecma262/#sec-numeric-types-number-add
                            // Implement to spec

                            // 2. NOTE: At this point, it must be a numeric operation.

                            //3. Let lNum be ? ToNumeric(lVal).
                            let left_numeric = Interpreter::to_numeric(left_primitive);

                            //4. Let rNum be ? ToNumeric(rVal).
                            let right_numeric = Interpreter::to_numeric(right_primitive);

                            // 5. If SameType(lNum, rNum) is false, throw a TypeError exception.
                            if !Interpreter::same_type(&left_numeric, &right_numeric) {
                                todo!("Throw TypeError exception");
                            }

                            // TODO: 6. If lNum is a BigInt, then

                            //7. Else,
                            match (left_numeric, right_numeric) {
                                (JSValue::Numeric(left_value), JSValue::Numeric(right_value)) => {
                                    return JSValue::Numeric(left_value + right_value);
                                },
                                _ => { panic!("Unexpected right JS value") }
                            }
                        }
                    }
                }
            }
        } else {
            // d. Set lVal to lPrim.
            // e. Set rVal to rPrim.
            let left_primitive = Interpreter::to_primitive(left, None);
            let right_primitive = Interpreter::to_primitive(right, None);

            match operator {
                // https://tc39.es/ecma262/#sec-numeric-types-number-multiply
                // TODO: Implement to spec
                TokenType::STAR => {
                    // 2. NOTE: At this point, it must be a numeric operation.

                    //3. Let lNum be ? ToNumeric(lVal).
                    let left_numeric = Interpreter::to_numeric(left_primitive);

                    //4. Let rNum be ? ToNumeric(rVal).
                    let right_numeric = Interpreter::to_numeric(right_primitive);

                    // 5. If SameType(lNum, rNum) is false, throw a TypeError exception.
                    if !Interpreter::same_type(&left_numeric, &right_numeric) {
                        todo!("Throw TypeError exception");
                    }

                    // TODO: 6. If lNum is a BigInt, then

                    //7. Else,
                    match (left_numeric, right_numeric) {
                        (JSValue::Numeric(left_value), JSValue::Numeric(right_value)) => {
                            return JSValue::Numeric(left_value * right_value);
                        },
                        _ => { panic!("Unexpected right JS value") }
                    }
                },
                // https://tc39.es/ecma262/#sec-numeric-types-number-divide
                // TODO: Implement to spec
                TokenType::SLASH => {
                    // 2. NOTE: At this point, it must be a numeric operation.

                    //3. Let lNum be ? ToNumeric(lVal).
                    let left_numeric = Interpreter::to_numeric(left_primitive);

                    //4. Let rNum be ? ToNumeric(rVal).
                    let right_numeric = Interpreter::to_numeric(right_primitive);

                    // 5. If SameType(lNum, rNum) is false, throw a TypeError exception.
                    if !Interpreter::same_type(&left_numeric, &right_numeric) {
                        todo!("Throw TypeError exception");
                    }

                    // TODO: 6. If lNum is a BigInt, then

                    //7. Else,
                    match (left_numeric, right_numeric) {
                        (JSValue::Numeric(left_value), JSValue::Numeric(right_value)) => {
                            return JSValue::Numeric(left_value / right_value);
                        },
                        _ => { panic!("Unexpected right JS value") }
                    }
                },
                // https://tc39.es/ecma262/#sec-numeric-types-number-subtract
                // Implement to spec
                TokenType::MINUS => {
                    // 2. NOTE: At this point, it must be a numeric operation.

                    //3. Let lNum be ? ToNumeric(lVal).
                    let left_numeric = Interpreter::to_numeric(left_primitive);

                    //4. Let rNum be ? ToNumeric(rVal).
                    let right_numeric = Interpreter::to_numeric(right_primitive);

                    // 5. If SameType(lNum, rNum) is false, throw a TypeError exception.
                    if !Interpreter::same_type(&left_numeric, &right_numeric) {
                        todo!("Throw TypeError exception");
                    }

                    // TODO: 6. If lNum is a BigInt, then

                    //7. Else,
                    match (left_numeric, right_numeric) {
                        (JSValue::Numeric(left_value), JSValue::Numeric(right_value)) => {
                            return JSValue::Numeric(left_value - right_value);
                        },
                        _ => { panic!("Unexpected right JSValue") }
                    }
                },
                _ => { panic!("Unexpected operator: {:?}", operator) }
            }
        }




    }

    // https://tc39.es/ecma262/#sec-tostring
    fn to_string(value: JSValue) -> JSValue {
        match value {
            // 1. If argument is a String, return argument.
            JSValue::String(value) => {
                return JSValue::String(value);
            },
            // 2. If argument is a Symbol, throw a TypeError exception.
            JSValue::Symbol(value) => {
                todo!("Throw a TypeError exception");
            },
            // 3. If argument is undefined, return "undefined".
            JSValue::Undefined => {
                return JSValue::String("undefined".to_string());
            }
            // 4. If argument is null, return "null".
            JSValue::Null => {
                return JSValue::String("null".to_string());
            },
            // 5. If argument is true, return "true".
            JSValue::Boolean(true) => {
                return JSValue::String("true".to_string());
            },
            // 6. If argument is false, return "false".
            JSValue::Boolean(false) => {
                return JSValue::String("false".to_string());
            },
            // 7. If argument is a Number, return Number::toString(argument, 10).
            JSValue::Numeric(value) => {
                return JSValue::String(Interpreter::number_to_string(value));
            },
            // 8. TODO: If argument is a BigInt, return BigInt::toString(argument, 10).

            // 9. Assert: argument is an Object.
            JSValue::Object(value) => {
                // 10. Let primValue be ? ToPrimitive(argument, string).
                // 11. Assert: primValue is not an Object.
                // 12. Return ? ToString(primValue).
                todo!();
            }
        }
    }

    // https://tc39.es/ecma262/#sec-numeric-types-number-tostring
    // TODO: Implement this to spec, for now we'll just use Rust's default implementation of to_string on numbers
    fn number_to_string(value: Number) -> String {
        return value.to_string();
    }

    // https://tc39.es/ecma262/#sec-sametype
    fn same_type(left: &JSValue, right: &JSValue) -> bool {

        match (left, right) {
            // 1. If x is undefined and y is undefined, return true.
            (JSValue::Undefined, JSValue::Undefined) => {
                return true;
            },
            // 2. If x is null and y is null, return true.
            (JSValue::Null, JSValue::Null) => {
                return true;
            },
            // 3. If x is a Boolean and y is a Boolean, return true.
            (JSValue::Boolean(_), JSValue::Boolean(_)) => {
                return true;
            },
            // 4. If x is a Number and y is a Number, return true.
            (JSValue::Numeric(_), JSValue::Numeric(_)) => {
                return true;
            },
            // 5. TODO:  If x is a BigInt and y is a BigInt, return true.

            // 6. If x is a Symbol and y is a Symbol, return true.
            (JSValue::Symbol(_), JSValue::Symbol(_)) => {
                return true;
            },
            // 7. If x is a String and y is a String, return true.
            (JSValue::String(_), JSValue::String(_)) => {
                return true;
            },
            // 8. If x is an Object and y is an Object, return true.
            (JSValue::Object(_), JSValue::Object(_)) => {
                return true;
            },
            // 9. Return false.
            _ => {
                return false;
            }
        }
    }

}
