use parser::Operator;
use parser::Token;

#[derive(Debug, PartialEq)]
pub enum Expression {
    Number(f32),
    String(String),
    Name(String),
    Binary(Operator, Box<Expression>, Box<Expression>),
    Unary(Operator, Box<Expression>),
    Call(String, Vec<Expression>),
    Invalid(String)
}

impl Expression {

    /// A Implementation of the Shunting Yard Algorithm
    pub fn new(tokens: Vec<Token>) -> Expression {

        let mut values: Vec<Expression> = vec![];
        let mut operators: Vec<Operator> = vec![];
        let mut valid_unary_position = false;
        let mut is_callable = false;

        for token in tokens {

            match token {

                // TODO handle macro args
                Token::Number(value) => {
                    values.push(Expression::Number(value));
                    is_callable = false;
                    valid_unary_position = false;
                },

                Token::String(string) => {
                    values.push(Expression::String(string));
                    is_callable = false;
                    valid_unary_position = false;
                },

                Token::LParen => {

                    if is_callable {
                        operators.push(Operator::Call);
                    }

                    operators.push(Operator::Paren);
                    is_callable = false;
                    valid_unary_position = true;

                },

                Token::Name(name) => {
                    values.push(Expression::Name(name));
                    is_callable = true;
                    valid_unary_position = false;
                },

                Token::Operator(mut op) => {

                    // Unary: Follows another operator or a left paren
                    if valid_unary_position {

                        // TODO check if we got a valid unary operator

                        // Replace normal minus with its unary equivalant
                        if op == Operator::Minus {
                            op = Operator::UnaryMinus;
                        }

                    // Binary: Follows an operand or a right paren
                    } else {
                        consume_operator(&mut values, &mut operators, op.get_prec());
                    }

                    operators.push(op);

                    is_callable = false;
                    valid_unary_position = true;

                },

                Token::RParen | Token::Comma => {

                    // Pop all operators until we find the matching open paren
                    while *operators.last().unwrap() != Operator::Paren {
                        consume_operator(&mut values, &mut operators, 0);
                    }

                    // Closing paren needs to pop the open paren
                    if token == Token::RParen {

                        // Pop open paren
                        operators.pop();

                        // See if the topmost operator is a call and build a
                        // call expression from it
                        if match operators.last() {
                            Some(op) => *op == Operator::Call,
                            None => false
                        } {

                            let mut args: Vec<Expression> = vec![];

                            // First pop call operator
                            operators.pop();

                            // Then pop values until we find the name of the
                            // function that is being called
                            while values.len() > 0 {

                                let value = values.pop().unwrap();
                                match value {

                                    // Found a name, so build a call expression
                                    Expression::Name(name) => {

                                        // Reverse the argument list
                                        args.reverse();

                                        values.push(Expression::Call(name, args));
                                        break;

                                    },

                                    // Any other value gets pushed into the arguments
                                    _ => {
                                        args.push(value);
                                    }

                                }

                            }

                        }

                    }

                    is_callable = false;
                    valid_unary_position = false;

                },

                _ => unreachable!()

            }
        }

        // Assert correct final state of the algorithm
        assert_eq!(operators.len(), 0);
        assert_eq!(values.len(), 1);

        // The last remaining value is the final expression tree
        values.pop().unwrap()

    }

}

fn consume_operator(values: &mut Vec<Expression>, operators: &mut Vec<Operator>, prec: i32) {

    // Check if the operator on top of the operator stack has a higher precedence
    if match operators.last() {
        // We leave equal precedence operators on the stack, this allows for right-associativity
        Some(op) => op.get_prec() > prec,
        None => false
    } {

        // Pop operator from stack
        let op = operators.pop().unwrap();

        // Get the right hand side operand
        let right = values.pop().unwrap();

        match op {

            // Create a unary expression
            Operator::UnaryMinus | Operator::UnaryNot => {
                values.push(Expression::Unary(op, Box::new(right)));
            },

            // Create a binary expression by popping the left operand from
            // the stack, if there is no other operand than we have an invalid
            // unary operation at hand
            // TODO fail early by returning from the expression
            // build when checking for unary expression types
            _ => {
                let operand = values.pop();
                values.push(match operand {
                    Some(left) => {
                        Expression::Binary(op, Box::new(left), Box::new(right))
                    },
                    None => {
                        Expression::Invalid("Invalid unary operator".to_string())
                    }
                });
            }

        };

    }

}

/*

use std::fmt;
use std::iter;

#[derive(PartialEq, Copy, Clone)]
enum Op {
    LogicalOr,
    LogicalAnd,
    BitwiseOr,
    BitwiseXor,
    BitwiseAnd,
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    LessThenEqual,
    GreaterThenEqual,
    ShiftLeft,
    ShiftRight,
    Plus,
    Minus,
    Not,
    Negate,
    Multiply,
    Divide,
    Modulo,
    Power
}

impl Op {

    pub fn prec(&self) -> i32 {
        match *self {
            Op::LogicalOr => 1,
            Op::LogicalAnd => 2,
            Op::BitwiseOr => 3,
            Op::BitwiseXor => 4,
            Op::BitwiseAnd => 5,
            Op::Equal => 6,
            Op::NotEqual => 6,
            Op::LessThan => 7,
            Op::GreaterThan => 7,
            Op::LessThenEqual => 7,
            Op::GreaterThenEqual => 7,
            Op::ShiftLeft => 8,
            Op::ShiftRight => 8,
            Op::Plus => 9,
            Op::Minus => 9,
            Op::Not => 9,
            Op::Negate => 9,
            Op::Multiply => 11,
            Op::Divide => 11,
            Op::Modulo => 11,
            Op::Power => 12
        }
    }

}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            Op::LogicalOr => "||",
            Op::LogicalAnd => "&&",
            Op::BitwiseOr => "|",
            Op::BitwiseXor => "^",
            Op::BitwiseAnd => "&",
            Op::Equal => "==",
            Op::NotEqual => "!=",
            Op::LessThan => "<",
            Op::GreaterThan => ">",
            Op::LessThenEqual => "<=",
            Op::GreaterThenEqual => ">=",
            Op::ShiftLeft => "<<",
            Op::ShiftRight => ">>",
            Op::Plus => "+",
            Op::Minus => "-",
            Op::Not => "!",
            Op::Negate => "~",
            Op::Multiply => "*",
            Op::Divide => "/",
            Op::Modulo => "%",
            Op::Power => "**"
        };
        write!(f, "{}", s)
    }

}

enum Exp {
    Number(f32),
    String(String),
    Name(String),
    Binary(Op, Box<Exp>, Box<Exp>),
    Unary(Op, Box<Exp>),
    Call(String, Vec<Exp>),
    Invalid(String)
}

fn resolve_name(name: String) -> Exp {
    Exp::Number(2.0)
}

fn resolve_and_call_macro(name: String, args: Vec<Exp>) -> Exp {
    Exp::Number(128.0)
}

fn bool_to_number(value: bool) -> Exp {
    if value == true {
        Exp::Number(1.0)

    } else {
        Exp::Number(0.0)
    }
}

fn evaluate_expression(expr: Exp) -> Exp {
    match expr {
        Exp::Number(value) => Exp::Number(value),
        Exp::String(value) => Exp::String(value),
        Exp::Name(value) => resolve_name(value),
        Exp::Binary(op, left, right) => {
            match (evaluate_expression(*left), evaluate_expression(*right)) {
                (Exp::Number(a), Exp::Number(b)) => {
                    match op {
                        Op::LogicalOr => bool_to_number(a != 0.0 || b != 0.0),
                        Op::LogicalAnd => bool_to_number(a != 0.0 && b != 0.0),
                        Op::BitwiseOr => Exp::Number((a as u32 | b as u32) as f32),
                        Op::BitwiseXor => Exp::Number((a as u32 ^ b as u32) as f32),
                        Op::BitwiseAnd => Exp::Number((a as u32 & b as u32) as f32),
                        Op::Equal => bool_to_number(a == b),
                        Op::NotEqual => bool_to_number(a != b),
                        Op::LessThan => bool_to_number(a < b),
                        Op::GreaterThan => bool_to_number(a > b),
                        Op::LessThenEqual => bool_to_number(a <= b),
                        Op::GreaterThenEqual => bool_to_number(a >= b),
                        Op::ShiftLeft => Exp::Number(((a as u32) << (b as u32)) as f32),
                        Op::ShiftRight => Exp::Number(((a as u32) >> (b as u32)) as f32),
                        Op::Plus => Exp::Number(a + b),
                        Op::Minus => Exp::Number(a - b),
                        Op::Multiply => Exp::Number(a * b),
                        Op::Divide => Exp::Number(a / b),
                        Op::Modulo => Exp::Number(a % b),
                        Op::Power => Exp::Number(a.powf(b)),
                        _ => Exp::Invalid(format!("Invalid binary operation: {} {} {}", a, op, b))
                    }
                },
                (Exp::String(a), Exp::String(b)) => {
                    match op {
                        Op::Equal => bool_to_number(a == b),
                        Op::NotEqual => bool_to_number(a != b),
                        Op::Plus => Exp::String(a + &b),
                        _ => Exp::Invalid(format!("Invalid binary operation: {} {} {}", a, op, b))
                    }
                },
                (Exp::String(a), Exp::Number(b)) => {
                    match op {
                        Op::Multiply => {
                            Exp::String(iter::repeat(&*a).take(b as usize).collect())
                        },
                        _ => Exp::Invalid(format!("Invalid binary operation: {} {} {}", a, op, b))
                    }
                },
                (Exp::Number(a), Exp::String(b)) => {
                    match op {
                        Op::Multiply => {
                            Exp::String(iter::repeat(&*b).take(a as usize).collect())
                        },
                        _ => Exp::Invalid(format!("Invalid binary operation: {} {} {}", a, op, b))
                    }
                },
                (_, _) => Exp::Invalid(format!("Invalid operands for binary operator {}", op))
            }
        },
        Exp::Unary(op, right) => {
            match evaluate_expression(*right) {
                Exp::Number(a) => {
                    match op {
                        Op::Minus => Exp::Number(-a),
                        Op::Negate => Exp::Number(!(a as i32) as f32),
                        Op::Not => bool_to_number(!(a == 1.0)),
                        _ => Exp::Invalid(format!("Invalid unary operation: {}{}", op, a))
                    }
                },
                Exp::String(a) => {
                    Exp::String("foo".to_owned())
                },
                _ => Exp::Invalid(format!("Invalid right hand-side value for unary operator {}", op))
            }
        },
        Exp::Call(name, args) => {
            resolve_and_call_macro(name, args.into_iter().map(
                |x| evaluate_expression(x)

            ).collect())
        },
        Exp::Invalid(err) => Exp::Invalid(err)
    }
}

enum Type {
    Exp(Exp),
    Number(f32),
    String(String)
}


/*
struct Token {
    typ: Type
}

#[derive(Debug)]
struct Foo {
    items: Vec<Item>
}

impl Foo {
    pub fn new() -> Foo {
        Foo {
            items: Vec::new()
        }
    }

    pub fn parse_bar(&mut self) {
        self.items.push(Item::new());
    }

    pub fn get_bar(&self) -> Bar {
        Bar::new(&self.items)
    }
}


#[derive(Debug, Copy, Clone)]
struct Item;

impl Item {
    pub fn new() -> Item {
        Item
    }
}


#[derive(Debug, Copy, Clone)]
struct Bar<'a> {
    pub items: &'a Vec<Item>
}

impl <'a>Bar<'a> {
    pub fn new(items: &'a Vec<Item>) -> Bar<'a> {
        Bar {
            items: items
        }
    }
}

fn get() -> Foo {
    let mut foo = Foo::new();
    foo.parse_bar();
    foo
}
*/

#[derive(PartialEq, Copy, Clone)]
enum Token {
    Eof,
    Comma,
    LParen,
    RParen,
    Number(f32),
    Operator(Op)
}

impl Token {
}

struct Expression {
    index: usize,
    tokens: Vec<Box<Token>>,
    op: Op,
    prec: i32,
    is_binary: bool,
    is_unary: bool,
    token: Token
}

impl Expression {

    pub fn new(tokens: Vec<Box<Token>>) -> Expression {
        Expression {
            index: 0,
            tokens: tokens,
            op: Op::Plus,
            prec: 0,
            is_binary: false,
            is_unary: false,
            token: Token::Eof
        }
    }

    pub fn parse_binary(&mut self, prec: i32) -> Exp {

        // Get first token
        self.next();

        // We always start with a unary expression
        let mut left = self.parse_unary();

        // Now we collect additional binary operators on right as long as their
        // precedence is higher then the initial one
        while self.is_binary && self.prec > prec {

            // Now we check it's associativity
            let prec = self.prec + match self.op {
                Op::BitwiseXor | Op::Power => 0,
                _ => 1
            };

            // Consume one token
            self.next();

            // And parse another binaryExpression to it's right
            let right = self.parse_binary(prec);

            // Then we combine our current expression with the operator and
            // the expression after it to a binary expression node
            left = Exp::Binary(self.op, Box::new(left), Box::new(right));

        }

        left

    }

    pub fn parse_unary(&mut self) -> Exp {

        // Unary expressions
        if self.is_unary {

            // Get operator and consume token
            self.next();

            // Return a new unary expression from the operator and a right hand side
            // binary expression
            let p = self.prec;
            Exp::Unary(self.op, Box::new(self.parse_binary(p)))

        } else {
            match self.token {

                // Parenthesis
                Token::LParen => {

                    // Consume token
                    self.next();

                    // Parse binary expression
                    let left = self.parse_binary(0);

                    // Expect a closing paren and return the expression inside
                    // the parenthesis
                    if self.expect(Token::RParen) {

                        // Consume closing paren
                        self.next();

                        left

                    } else {
                        Exp::Invalid("Unbalanced parenthesis".to_owned())
                    }

                },

                // Invalid
                Token::RParen | Token::Operator(_) | Token::Comma => {
                    Exp::Invalid("Invalid token".to_owned())
                },

                // Values / Calls
                _ => {

                    // Names can be part of macro calls
                    Expression::Invalid("foo".to_owned())

                }

            }
        }

    }

    fn next(&mut self) {
        match self.tokens.get(self.index) {
            None => {
                self.prec = 0;
            },
            Some(token) => {

                match **token {
                    Token::Operator(op) => {

                        self.op = op;
                        self.prec = op.prec();
                        self.is_binary = match op {
                            Op::Not | Op::Negate => false,
                            _ => true
                        };

                        self.is_unary = match op {
                            Op::Minus | Op::Not | Op::Negate | Op::Plus => true,
                            _ => false
                        };

                    },
                    _ => {
                        self.prec = 0;
                        self.is_binary = false;
                        self.is_unary = false;
                    }
                };

                self.token = **token;
                self.index += 1;

            }
        }
    }

    fn expect(&self, token: Token) -> bool {
        self.token == token
    }

}
*/
