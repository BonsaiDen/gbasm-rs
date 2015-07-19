use parser::Operator;

#[derive(Debug, PartialEq)]
pub enum Token {
    Newline,
    Whitespace,
    Comment(String),
    String(String),
    Directive(String),
    Instruction(String),
    Name(String),
    Number(f32),
    Operator(Operator),
    GlobalLabelDef(String),
    LocalLabelDef(String),
    LocalLabelRef(String),
    Error(String),
    MacroArg(String),
    MacroDef,
    MacroEnd,
    NegativeOffset,
    PositiveOffset,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Eof,
}

