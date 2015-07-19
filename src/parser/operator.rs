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

