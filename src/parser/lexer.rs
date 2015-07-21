use std::iter;

use parser::Operator;
use parser::Token;
use parser::TokenType;
use parser::Expression;
use compiler::SourceIter;

/// Assembly Tokenizer which already builts expression trees
pub struct Lexer<'a> {
    lexer: iter::Peekable<BaseLexer<'a>>,
    in_macro_args: bool,
    in_macro_body: bool,
    paren_depth: u8,
    last_token_type: TokenType
}

impl <'a>Lexer<'a> {

    pub fn new(source: &'a mut SourceIter) -> Lexer<'a> {
        Lexer {
            lexer: BaseLexer::new(source).peekable(),
            in_macro_args: false,
            in_macro_body: false,
            paren_depth: 0,
            last_token_type: TokenType::Begin
        }
    }

    fn next_token(&mut self) -> Token {
        self.lexer.next().unwrap()
    }

}

impl<'a> Iterator for Lexer<'a> {

    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {

        let token = match self.next_token() {

            // Combine offset labels with their argument
            Token::PositiveOffset => {
                match self.next_token() {
                    Token::Number(val) => Token::Offset(val as i32),
                    _ => Token::Error("Expected number after offset sign".to_string())
                }
            },

            Token::NegativeOffset => {
                match self.next_token() {
                    Token::Number(val) => Token::Offset(-(val as i32)),
                    _ => Token::Error("Expected number after offset sign".to_string())
                }
            },

            // Disallow macro args outside of macro signatures and bodies
            Token::MacroArg(name) => {
                if !self.in_macro_args && !self.in_macro_body {
                    Token::Error(format!("Unexpected MarcoArg @{} outside of marco arguments or macro body", name))

                } else {
                    Token::MacroArg(name.to_owned())
                }
            },

            // Combine macro tokens with their name
            Token::MacroDef => {
                match self.next_token() {
                    Token::Name(name) => {
                        if self.in_macro_args {
                            Token::Error("Already inside a MACRO arguments signature".to_string())

                        } else {
                            self.in_macro_args = true;
                            Token::Macro(name)
                        }
                    },
                    _ => Token::Error("Expected name after MARCO directive".to_string())
                }
            },

            // End Macro bodies
            token @ Token::MacroEnd => {
                if !self.in_macro_body {
                    Token::Error("Unexpected MARCO_END directive outside of macro".to_string())

                } else {
                    self.in_macro_body = false;
                    token
                }
            },

            // Find and build expressions
            token => {

                // Wait for macro argument definitions to close
                if self.in_macro_args {
                    if token == Token::RParen {
                        self.in_macro_args = false;
                        self.in_macro_body = true;
                    }
                    token

                } else {

                    // Collect expression tokens, wrapping the stack in
                    // parenthesis for easier parsing
                    let mut token_type = token.to_type();

                    if is_expression(&self.last_token_type, &token_type, self.paren_depth) {

                        // Start expression stack
                        let mut expression_stack = vec![Token::LParen, token];

                        loop {

                            // Handle parenthesis nesting
                            match token_type {
                                TokenType::LParen => self.paren_depth += 1,
                                TokenType::RParen => self.paren_depth -= 1,
                                _ => {}
                            };

                            // Remember last token type
                            self.last_token_type = token_type;

                            // Peek next token type
                            token_type = match self.lexer.peek() {
                                Some(token) => token.to_type(),
                                None => TokenType::Eof
                            };

                            // Check if the expression continues
                            if is_expression(&self.last_token_type, &token_type, self.paren_depth) {
                                expression_stack.push(self.next_token());

                            } else {
                                break
                            }

                        }

                        expression_stack.push(Token::RParen);
                        Token::Expression(Expression::new(expression_stack))

                    } else {
                        token
                    }

                }

            }

        };

        self.last_token_type = token.to_type();
        Some(token)

    }

}

struct BaseLexer<'a> {
    source: &'a mut SourceIter
}


impl<'a> Iterator for BaseLexer<'a> {

    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.next_raw_token() {
                Token::Whitespace | Token::Comment(_) => {
                    continue;
                },
                token => return Some(token)
            }
        }
    }

}

impl <'a>BaseLexer<'a> {

    pub fn new(source: &'a mut SourceIter) -> BaseLexer<'a> {

        // Goto first byte in iterator
        source.next();

        BaseLexer {
            source: source
        }

    }

    fn next_raw_token(&mut self) -> Token {

        let ch = self.source.get();
        let next = self.source.peek();

        match ch {

            // Newlines
            b'\r' | b'\n' => {
                self.source.next();
                Token::Newline
            },

            // Parse Comments
            b';' => self.parse_comment(),

            // Parse Parenthesis and Commas
            b'(' => {
                self.source.next();
                Token::LParen
            }
            b')' => {
                self.source.next();
                Token::RParen
            },
            b'[' => {
                self.source.next();
                Token::LBrace
            }
            b']' => {
                self.source.next();
                Token::RBrace
            }
            b',' => {
                self.source.next();
                Token::Comma
            }

            // Parse Strings
            b'"' | b'\'' => self.parse_string(),

            // Parse relative Address offsets and Macro Arguments
            b'@' => self.parse_offset_or_macro_arg(),

            // Parse negative Decimal Numbers
            b'-' if is_decimal(next) => {
                self.source.next();
                self.parse_decimal(true)
            }

            // Parse Binary Numbers
            b'%' if is_binary(next) => {
                self.source.next();
                self.parse_binary()
            }

            // Parse Hexadecimal Numbers
            b'$' if is_hex(next) => {
                self.source.next();
                self.parse_hex()
            }

            // Parse local Labels
            b'.' if is_name_start(next) => self.parse_local_label(),

            // Skip Whitespace
            _ if is_whitespace(ch) => self.parse_whitespace(),

            // Parse Names and global Labels
            _ if is_name_start(ch) => self.parse_name(),

            // Parse positive Decimal Numbers
            _ if is_decimal(ch) => self.parse_decimal(false),

            // Parse Operators
            _ if is_operator(ch) => self.parse_operator(ch, next),

            // End of File
            0 => Token::Eof,

            // Unkown symbols
            _ => Token::Error(format!("Unexpected character \"{}\" ({})", "", ch))

        }

    }

    fn parse_comment(&mut self) -> Token {

        let mut bytes = vec![];

        self.source.next();
        while !self.source.is_empty() && !is_newline(self.source.get()) {
            bytes.push(self.source.get());
            self.source.next();
        }

        Token::Comment(string_from_bytes(bytes))

    }

    fn parse_whitespace(&mut self) -> Token {
        while !self.source.is_empty() && is_whitespace(self.source.get()) {
            self.source.next();
        }
        Token::Whitespace
    }

    fn parse_string(&mut self) -> Token {

        let delimiter = self.source.get();
        let mut bytes: Vec<u8> = Vec::new();
        let mut ch: u8 = self.source.next();

        while ch != delimiter && !self.source.is_empty() {

            // Escape sequences
            if ch == b'\\' {
                bytes.push(match self.source.next() {
                    b'0' => 0,
                    b'b' => 7,
                    b't' => 9,
                    b'n' => 10,
                    b'v' => 11,
                    b'r' => 13,
                    b'"' => 34,
                    b'\'' => 39,
                    b'\\' => 92,
                    c => {
                        return Token::Error(format!("Unkown character escape sequence \"{}\" in string literal", c))
                    }
                });

            } else {
                bytes.push(ch);
            }

            ch = self.source.next();

        }

        if ch != delimiter {
            Token::Error("Unclosed string literal".to_string())

        } else {
            self.source.next();
            match String::from_utf8(bytes) {
                Ok(v) => Token::String(v),
                Err(_) => Token::Error("Invalid string literal contents".to_string())
            }
        }

    }

    fn parse_operator(&mut self, ch: u8, next: u8) -> Token {

        self.source.next();

        match (ch, next) {

            // Double Character Operators
            (b'=', b'=') => {
                self.source.next();
                Token::Operator(Operator::Equal)
            }
            (b'>', b'>') => {
                self.source.next();
                Token::Operator(Operator::ShiftRight)
            }
            (b'<', b'<') => {
                self.source.next();
                Token::Operator(Operator::ShiftLeft)
            }
            (b'&', b'&') => {
                self.source.next();
                Token::Operator(Operator::LogicalAnd)
            }
            (b'|', b'|') => {
                self.source.next();
                Token::Operator(Operator::LogicalOr)
            }
            (b'!', b'=') => {
                self.source.next();
                Token::Operator(Operator::NotEqual)
            }
            (b'>', b'=') => {
                self.source.next();
                Token::Operator(Operator::GreaterThanEqual)
            }
            (b'<', b'=') => {
                self.source.next();
                Token::Operator(Operator::LessThanEqual)
            }
            (b'/', b'/') => {
                self.source.next();
                Token::Operator(Operator::IntegerDivide)
            }
            (b'*', b'*') => {
                self.source.next();
                Token::Operator(Operator::Power)
            }

            // Single Character Operatots
            (_, _) => match ch {
                b'>' => Token::Operator(Operator::GreaterThan),
                b'<' => Token::Operator(Operator::LessThan),
                b'!' => Token::Operator(Operator::UnaryNot),
                b'+' => Token::Operator(Operator::Plus),
                b'-' => Token::Operator(Operator::Minus),
                b'*' => Token::Operator(Operator::Multiply),
                b'/' => Token::Operator(Operator::Divide),
                b'%' => Token::Operator(Operator::Modulo),
                b'&' => Token::Operator(Operator::BitwiseAnd),
                b'|' => Token::Operator(Operator::BitwiseOr),
                b'~' => Token::Operator(Operator::Negate),
                b'^' => Token::Operator(Operator::BitwiseXor),
                _ => Token::Error(format!("Invalid operator \"{}\"", ch))
            }

        }

    }

    fn parse_decimal(&mut self, is_negative: bool) -> Token {

        let (digit, number, len) = self.parse_decimal_part();

        if len == 8 {
            Token::Error("Decimal literal exceeds maximum length of 8 digits".to_string())

        // Floats
        } else if digit == b'.' {

            self.source.next();

            let (_, float_digits, prec) = self.parse_decimal_part();
            let float = number as f32 + (
                float_digits as f32 / (10 as f32).powf(prec as f32)
            );

            Token::Number(if is_negative { -float } else { float })

        // Integers
        } else {
            Token::Number((if is_negative { -number } else { number }) as f32)
        }

    }

    fn parse_decimal_part(&mut self) -> (u8, i32, usize) {

        let mut digit = self.source.get();
        let mut bytes: Vec<u8> = Vec::new();

        while is_decimal(digit) {

            bytes.push(digit - b'0');
            digit = self.source.next();

            // Ignore interleaved underscore characters
            if digit == b'_' {
                digit = self.source.next();

            } else if bytes.len() == 8 {
                break;
            }

        }

        (digit, to_number(&bytes, 10), bytes.len())

    }

    fn parse_binary(&mut self) -> Token {

        let mut digit = self.source.get();
        let mut bytes: Vec<u8> = Vec::new();

        while is_binary(digit) {

            bytes.push(digit - b'0');
            digit = self.source.next();

            // Ignore interleaved underscore characters
            if digit == b'_' {
                digit = self.source.next();

            } else if bytes.len() == 9 {
                return Token::Error("Binary literal exceeds maximum length of 8 digits".to_string());
            }

        }

        Token::Number(to_number(&bytes, 2) as f32)

    }

    fn parse_hex(&mut self) -> Token {

        let mut digit = self.source.get();
        let mut bytes: Vec<u8> = Vec::new();

        while is_hex(digit) {

            if digit >= b'a' {
                digit -= 87;

            } else if digit >= b'A' {
                digit -= 55;

            } else {
                digit -= b'0';
            }

            bytes.push(digit);
            digit = self.source.next();

            // Ignore interleaved underscore characters
            if digit == b'_' {
                digit = self.source.next();

            } else if bytes.len() == 5 {
                return Token::Error("Hex literal exceeds maximum length of 4 digits".to_string());
            }

        }

        Token::Number(to_number(&bytes, 16) as f32)

    }

    fn parse_name(&mut self) -> Token {

        let mut bytes: Vec<u8> = Vec::new();
        let mut ch = self.source.get();

        while is_name_part(ch) {
            bytes.push(ch);
            ch = self.source.next();
        }

        let name = string_from_bytes(bytes);

        if is_instruction(&name[..]) {
            Token::Instruction(name)

        } else if is_directive(&name[..]) {
            Token::Directive(name)

        } else if name == "MACRO" {
            Token::MacroDef

        } else if name == "ENDMACRO" {
            Token::MacroEnd

        } else if name != "" {

            // Global Label Definitions
            if ch == b':' {
                self.source.next();
                Token::GlobalLabelDef(name)

            } else {
                Token::Name(name)
            }

        } else {
            Token::Error(format!("Unexpected empty name"))
        }

    }

    fn parse_local_label(&mut self) -> Token {

        let mut bytes: Vec<u8> = vec![self.source.get()];
        let mut ch = self.source.next();

        while is_name_part(ch) {
            bytes.push(ch);
            ch = self.source.next();
        }

        // Label Definition
        if ch == b':' {
            self.source.next();
            Token::LocalLabelDef(string_from_bytes(bytes))

        // Label Reference
        } else {
            Token::LocalLabelRef(string_from_bytes(bytes))
        }

    }

    fn parse_offset_or_macro_arg(&mut self) -> Token {

        let sign = self.source.next();

        // Negative Offset
        if sign == b'-' {
            self.source.next();
            Token::NegativeOffset

        // Positive Offset
        } else if sign == b'+' {
            self.source.next();
            Token::PositiveOffset

        // Macro Arguments
        } else if is_name_start(sign) {

            let mut bytes: Vec<u8> = vec![sign];
            let mut ch = self.source.next();

            while is_name_part(ch) {
                bytes.push(ch);
                ch = self.source.next();
            }

            Token::MacroArg(string_from_bytes(bytes))

        } else{
            Token::Error(format!("Unexpected \"{}\", expected a valid direction specifier (- or +) instead", sign))
        }

    }

}


// Helpers --------------------------------------------------------------------
fn string_from_bytes(bytes: Vec<u8>) -> String {
    match String::from_utf8(bytes) {
        Ok(v) => v,
        Err(_) => String::new()
    }
}

fn to_number(bytes: &Vec<u8>, radix: i32) -> i32 {

    let l = bytes.len() as u32;
    let mut num: i32 = 0;

    for i in 0..l as u32 {
        let c = bytes.get(i as usize);
        num += match c {
            Some(v) => radix.pow(l - i - 1) * (*v as i32),
            None => 0
        }
    }

    num

}


// Matchers -------------------------------------------------------------------
fn is_instruction(name: &str) -> bool {
    match name {
        "cp" => true,
        "di" => true,
        "ei" => true,
        "jp" => true,
        "jr" => true,
        "or" => true,
        "rl" => true,
        "rr" => true,
        "ld" => true,

        "adc" => true,
        "add" => true,
        "and" => true,
        "bit" => true,
        "ccf" => true,
        "cpl" => true,
        "daa" => true,
        "dec" => true,
        "inc" => true,
        "ldh" => true,
        "nop" => true,
        "pop" => true,
        "res" => true,
        "ret" => true,
        "rla" => true,
        "rlc" => true,
        "rra" => true,
        "rrc" => true,
        "rst" => true,
        "sbc" => true,
        "scf" => true,
        "set" => true,
        "sla" => true,
        "sra" => true,
        "srl" => true,
        "sub" => true,
        "xor" => true,

        "halt" => true,
        "push" => true,
        "call" => true,
        "reti" => true,
        "ldhl" => true,
        "rlca" => true,
        "rrca" => true,
        "stop" => true,
        "swap" => true,

        _ => false
    }
}

fn is_directive(name: &str) -> bool {
    match name {
        "DB" => true,
        "DW" => true,
        "DS" => true,

        "EQU" => true,
        "EQUS" => true,

        "BANK" => true,

        "INCBIN" => true,

        "SECTION" => true,
        "INCLUDE" => true,

        _ => false
    }
}

fn is_name_start(c: u8) -> bool {
    match c {
        b'A'...b'Z' => true,
        b'_' => true,
        b'a'...b'z' => true,
        _ => false
    }
}

fn is_name_part(c: u8) -> bool {
    is_name_start(c) || is_decimal(c)
}

fn is_decimal(c: u8) -> bool {
    match c {
        b'0'...b'9' => true,
        _ => false
    }
}

fn is_binary(c: u8) -> bool {
    match c {
        b'0' => true,
        b'1' => true,
        _ => false
    }
}

fn is_hex(c: u8) -> bool {
    match c {
        b'a'...b'f' => true,
        b'A'...b'F' => true,
        _ => is_decimal(c)
    }
}

fn is_newline(c: u8) -> bool {
    match c {
        b'\r' => true,
        b'\n' => true,
        _ => false
    }
}

fn is_whitespace(c: u8) -> bool {
    match c {
        9 => true,
        11 => true,
        b' ' => true,
        _ => false
    }
}

fn is_operator(c: u8) -> bool {
    match c {
        b'!' => true,
        b'%' => true,
        b'&' => true,
        b'*' => true,
        b'+' => true,
        b'-' => true,
        b'/' => true,
        b'<' => true,
        b'=' => true,
        b'>' => true,
        b'^' => true,
        b'|' => true,
        b'~' => true,
        _ => false
    }
}

fn is_expression(last: &TokenType, next: &TokenType, depth: u8) -> bool {

    match (last, next) {

        // Commas always separate expressions when outside of parenthesis
        (&TokenType::Comma, _) if depth == 0 => false,
        (_, &TokenType::Comma) if depth == 0 => false,

        // Left Parenthesis
        (&TokenType::LParen, &TokenType::Name) => true,
        (&TokenType::LParen, &TokenType::LocalLabelRef) => true,
        (&TokenType::LParen, &TokenType::Number) => true,
        (&TokenType::LParen, &TokenType::String) => true,
        (&TokenType::LParen, &TokenType::Operator) => true,
        (&TokenType::LParen, &TokenType::LParen) => true,
        (&TokenType::LParen, &TokenType::RParen) => true,
        (&TokenType::LParen, &TokenType::MacroArg) => true,

        // Right Parenthesis
        (&TokenType::RParen, &TokenType::RParen) => true,
        (&TokenType::RParen, &TokenType::Operator) => true,

        // Operators
        (&TokenType::Operator, &TokenType::LParen) => true,
        (&TokenType::Operator, &TokenType::Number) => true,
        (&TokenType::Operator, &TokenType::String) => true,
        (&TokenType::Operator, &TokenType::LocalLabelRef) => true,
        (&TokenType::Operator, &TokenType::Name) => true,
        (&TokenType::Operator, &TokenType::MacroArg) => true,

        // Numbers
        (&TokenType::Number, &TokenType::RParen) => true,
        (&TokenType::Number, &TokenType::Operator) => true,
        (&TokenType::Number, &TokenType::Comma) => true,

        // Strings
        (&TokenType::String, &TokenType::RParen) => true,
        (&TokenType::String, &TokenType::Operator) => true,
        (&TokenType::String, &TokenType::Comma) => true,

        // LocalLabelRef
        (&TokenType::LocalLabelRef, &TokenType::RParen) => true,
        (&TokenType::LocalLabelRef, &TokenType::Operator) => true,
        (&TokenType::LocalLabelRef, &TokenType::Comma) => true,

        // Names
        (&TokenType::Name, &TokenType::LParen) => true,
        (&TokenType::Name, &TokenType::RParen) => true,
        (&TokenType::Name, &TokenType::Operator) => true,
        (&TokenType::Name, &TokenType::Comma) => true,

        // Macro Args
        (&TokenType::MacroArg, &TokenType::LParen) => true,
        (&TokenType::MacroArg, &TokenType::RParen) => true,
        (&TokenType::MacroArg, &TokenType::Operator) => true,
        (&TokenType::MacroArg, &TokenType::Comma) => true,

        // Comma
        (&TokenType::Comma, &TokenType::LParen) => true,
        (&TokenType::Comma, &TokenType::Name) => true,
        (&TokenType::Comma, &TokenType::String) => true,
        (&TokenType::Comma, &TokenType::Number) => true,
        (&TokenType::Comma, &TokenType::MacroArg) => true,

        // Directive Values
        (&TokenType::Directive, &TokenType::LParen) => true,
        (&TokenType::Directive, &TokenType::Name) => true,
        (&TokenType::Directive, &TokenType::String) => true,
        (&TokenType::Directive, &TokenType::Number) => true,
        (&TokenType::Directive, &TokenType::MacroArg) => true,

        // Instruction Values
        (&TokenType::Instruction, &TokenType::LParen) => true,
        (&TokenType::Instruction, &TokenType::Name) => true,
        (&TokenType::Instruction, &TokenType::String) => true,
        (&TokenType::Instruction, &TokenType::Number) => true,
        (&TokenType::Instruction, &TokenType::MacroArg) => true,

        // Everything else
        (_, _) => false

    }
}

