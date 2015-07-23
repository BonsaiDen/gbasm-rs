use parser::Operator;
use parser::Expression;

#[derive(Debug, PartialEq)]
pub enum Token {
    Newline,
    Whitespace,
    Comment(String),
    String(String),
    Directive(String),
    Instruction(String),
    Expression(Expression),
    Name(String),
    Number(f32),
    Operator(Operator),
    GlobalLabelDef(String),
    LocalLabelDef(String),
    LocalLabelRef(String),
    Offset(i32),
    Error(String),
    Macro(String),
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
    Eof
}

impl Token {
    pub fn to_type(&self) -> TokenType {
        match self {
            &Token::Newline => TokenType::Newline,
            &Token::Whitespace => TokenType::Whitespace,
            &Token::Comment(_) => TokenType::Comment,
            &Token::String(_) => TokenType::String,
            &Token::Directive(_) => TokenType::Directive,
            &Token::Instruction(_) => TokenType::Instruction,
            &Token::Expression(_) => TokenType::Expression,
            &Token::Name(_) => TokenType::Name,
            &Token::Number(_) => TokenType::Number,
            &Token::Operator(_) => TokenType::Operator,
            &Token::GlobalLabelDef(_) => TokenType::GlobalLabelDef,
            &Token::LocalLabelDef(_) => TokenType::LocalLabelDef,
            &Token::LocalLabelRef(_) => TokenType::LocalLabelRef,
            &Token::Offset(_) => TokenType::Offset,
            &Token::Error(_) => TokenType::Error,
            &Token::Macro(_) => TokenType::Macro,
            &Token::MacroArg(_) => TokenType::MacroArg,
            &Token::MacroDef => TokenType::MacroDef,
            &Token::MacroEnd => TokenType::MacroEnd,
            &Token::NegativeOffset => TokenType::NegativeOffset,
            &Token::PositiveOffset => TokenType::PositiveOffset,
            &Token::LParen => TokenType::LParen,
            &Token::RParen => TokenType::RParen,
            &Token::LBrace => TokenType::LBrace,
            &Token::RBrace => TokenType::RBrace,
            &Token::Comma => TokenType::Comma,
            &Token::Eof => TokenType::Eof
        }
    }
}

#[derive(Copy, Clone)]
pub enum TokenType {
    Newline,
    Whitespace,
    Comment,
    String,
    Directive,
    Instruction,
    Expression,
    Name,
    Number,
    Operator,
    GlobalLabelDef,
    LocalLabelDef,
    LocalLabelRef,
    Offset,
    Error,
    Macro,
    MacroArg,
    MacroDef,
    MacroEnd,
    NegativeOffset,
    PositiveOffset,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Begin,
    Eof
}

