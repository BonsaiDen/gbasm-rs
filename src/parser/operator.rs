#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Operator {
    Paren,
    Call,
    LogicalOr,
    LogicalAnd,
    BitwiseOr,
    BitwiseXor,
    BitwiseAnd,
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    LessThanEqual,
    GreaterThanEqual,
    ShiftLeft,
    ShiftRight,
    Plus,
    Minus,
    Negate,
    Multiply,
    Divide,
    Modulo,
    Power,
    IntegerDivide,
    UnaryNot,
    UnaryMinus
}

impl Operator {

    /// Returns the operators precedence
    pub fn get_prec(&self) -> i32 {
        match *self {
            Operator::Paren => 0,
            Operator::Call => 0,
            Operator::LogicalOr => 1,
            Operator::LogicalAnd => 2,
            Operator::BitwiseOr => 3,
            Operator::BitwiseXor => 4,
            Operator::BitwiseAnd => 5,
            Operator::Equal => 6,
            Operator::NotEqual => 6,
            Operator::LessThan => 7,
            Operator::GreaterThan => 7,
            Operator::LessThanEqual => 7,
            Operator::GreaterThanEqual => 7,
            Operator::ShiftLeft => 8,
            Operator::ShiftRight => 8,
            Operator::Plus => 9,
            Operator::Minus => 9,
            Operator::Negate => 9,
            Operator::Multiply => 11,
            Operator::Divide => 11,
            Operator::IntegerDivide => 11,
            Operator::Modulo => 11,
            Operator::UnaryNot => 12,
            Operator::UnaryMinus => 12,
            Operator::Power => 13
        }
    }

}

