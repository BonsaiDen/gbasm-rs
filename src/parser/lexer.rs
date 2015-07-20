use parser::Operator;
use parser::Token;
use parser::TokenType;
use compiler::SourceIter;

pub struct Lexer<'a> {
    source: &'a mut SourceIter,
    in_macro_args: bool,
    in_macro_body: bool,
    paren_depth: u8,
    last_token_type: TokenType
}

impl <'a>Lexer<'a> {

    pub fn new(source: &'a mut SourceIter) -> Lexer<'a> {

        // Goto first byte in iterator
        source.next();

        Lexer {
            source: source,
            in_macro_args: false,
            in_macro_body: false,
            paren_depth: 0,
            last_token_type: TokenType::Begin
        }

    }

    pub fn next(&mut self) -> Token {

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
            mut token => {

                // Wait for macro argument definitions to close
                if self.in_macro_args {
                    if token == Token::RParen {
                        self.in_macro_args = false;
                        self.in_macro_body = true;
                    }
                    token

                } else {

                    // Collect expression tokens
                    let mut expression_stack = vec![];
                    while is_expression(&self.last_token_type, &token.to_type(), self.paren_depth) {
                        self.last_token_type = token.to_type();
                        expression_stack.push(token);
                        token = self.next_token();
                    }

                    if expression_stack.len() > 0 {
                        println!("expression stack: {:?}", expression_stack);
                        Token::Expression

                    } else {
                        token
                    }

                }

            }

        };

        self.last_token_type = token.to_type();
        token

    }

    fn next_token(&mut self) -> Token {
        loop {
            match self.next_raw_token() {
                Token::Whitespace | Token::Comment(_) => {
                    continue;
                },
                token => return token
            }
        }
    }

    fn next_raw_token(&mut self) -> Token {

        let ch: u8 = self.source.get();
        let next: u8 = self.source.peek();

        match ch {

            // Newlines
            10 | 13 => {
                self.source.next();
                Token::Newline
            },

            // Parse Comments
            59 => self.parse_comment(),

            // Parse Parenthesis and Commas
            40 => {
                self.source.next();
                Token::LParen
            }
            41 => {
                self.source.next();
                Token::RParen
            },
            91 => {
                self.source.next();
                Token::LBrace
            },
            93 => {
                self.source.next();
                Token::RBrace
            },
            44 => {
                self.source.next();
                Token::Comma
            },

            // Parse Strings
            34 | 39 => self.parse_string(),

            // Parse relative Address offsets and Macro Arguments
            64 => self.parse_offset_or_macro_arg(),

            // Parse negative Decimal Numbers
            45 if is_decimal(next) => {
                self.source.next();
                self.parse_decimal(true)
            },

            // Parse Binary Numbers
            37 if is_binary(next) => {
                self.source.next();
                self.parse_binary()
            },

            // Parse Hexadecimal Numbers
            36 if is_hex(next) => {
                self.source.next();
                self.parse_hex()
            },

            // Parse local Labels
            46 if is_name_start(next) => self.parse_local_label(),

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

        Token::Comment(to_string(bytes))

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
            if ch == 92 {
                bytes.push(match self.source.next() {
                    34 => 34,
                    39 => 39,
                    48 => 0,
                    98 => 7,
                    92 => 92,
                    110 => 10,
                    114 => 13,
                    118 => 11,
                    116 => 9,
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
            (61, 61) => {
                self.source.next();
                Token::Operator(Operator::Equal)
            }
            (62, 62) => {
                self.source.next();
                Token::Operator(Operator::ShiftRight)
            }
            (60, 60) => {
                self.source.next();
                Token::Operator(Operator::ShiftLeft)
            }
            (38, 38) => {
                self.source.next();
                Token::Operator(Operator::LogicalAnd)
            }
            (124, 124) => {
                self.source.next();
                Token::Operator(Operator::LogicalOr)
            }
            (33, 61) => {
                self.source.next();
                Token::Operator(Operator::NotEqual)
            }
            (62, 61) => {
                self.source.next();
                Token::Operator(Operator::GreaterThanEqual)
            }
            (60, 61) => {
                self.source.next();
                Token::Operator(Operator::LessThanEqual)
            }
            (47, 47) => {
                self.source.next();
                Token::Operator(Operator::IntegerDivide)
            }
            (42, 42) => {
                self.source.next();
                Token::Operator(Operator::Power)
            }

            // Single Character Operatots
            (_, _) => match ch {
                60 => Token::Operator(Operator::GreaterThan),
                62 => Token::Operator(Operator::LessThan),
                33 => Token::Operator(Operator::UnaryNot),
                43 => Token::Operator(Operator::Plus),
                45 => Token::Operator(Operator::Minus),
                42 => Token::Operator(Operator::Multiply),
                47 => Token::Operator(Operator::Divide),
                37 => Token::Operator(Operator::Modulo),
                38 => Token::Operator(Operator::BitwiseAnd),
                124 => Token::Operator(Operator::BitwiseOr),
                126 => Token::Operator(Operator::Negate),
                94 => Token::Operator(Operator::BitwiseXor),
                _ => Token::Error(format!("Invalid operator \"{}\"", ch))
            }

        }

    }

    fn parse_decimal(&mut self, is_negative: bool) -> Token {

        let (digit, number, len) = self.parse_decimal_part();

        if len == 8 {
            Token::Error("Decimal literal exceeds maximum length of 8 digits".to_string())

        // Floats
        } else if digit == 46 {

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

        while !self.source.is_empty() && is_decimal(digit) {

            bytes.push(digit - 48);
            digit = self.source.next();

            // Ignore interleaved underscore characters
            if digit == 95 {
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

        while !self.source.is_empty() && is_binary(digit) {

            bytes.push(digit - 48);
            digit = self.source.next();

            // Ignore interleaved underscore characters
            if digit == 95 {
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

        while !self.source.is_empty() && is_hex(digit) {

            if digit >= 97 {
                digit -= 87;

            } else if digit >= 65 {
                digit -= 55;

            } else {
                digit -= 48;
            }

            bytes.push(digit);
            digit = self.source.next();

            // Ignore interleaved underscore characters
            if digit == 95 {
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

        while !self.source.is_empty() && is_name_part(ch) {
            bytes.push(ch);
            ch = self.source.next();
        }

        let name = to_string(bytes);

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
            if ch == 58 {
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

        while !self.source.is_empty() && is_name_part(ch) {
            bytes.push(ch);
            ch = self.source.next();
        }

        // Label Definition
        if ch == 58 {
            self.source.next();
            Token::LocalLabelDef(to_string(bytes))

        // Label Reference
        } else {
            Token::LocalLabelRef(to_string(bytes))
        }

    }

    fn parse_offset_or_macro_arg(&mut self) -> Token {

        let sign = self.source.next();

        // Negative Offset
        if sign == 45 {
            self.source.next();
            Token::NegativeOffset

        // Positive Offset
        } else if sign == 43 {
            self.source.next();
            Token::PositiveOffset

        // Macro Arguments
        } else if is_name_start(sign) {

            let mut bytes: Vec<u8> = vec![sign];
            let mut ch = self.source.next();

            while !self.source.is_empty() && is_name_part(ch) {
                bytes.push(ch);
                ch = self.source.next();
            }

            Token::MacroArg(to_string(bytes))

        } else{
            Token::Error(format!("Unexpected \"{}\", expected a valid direction specifier (- or +) instead", sign))
        }

    }

}


// Helpers --------------------------------------------------------------------
fn to_string(bytes: Vec<u8>) -> String {
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
        65...90 => true,
        95 => true,
        97...122 => true,
        _ => false
    }
}

fn is_name_part(c: u8) -> bool {
    is_name_start(c) || is_decimal(c)
}

fn is_decimal(c: u8) -> bool {
    match c {
        48...57 => true,
        _ => false
    }
}

fn is_binary(c: u8) -> bool {
    match c {
        48 => true,
        49 => true,
        _ => false
    }
}

fn is_hex(c: u8) -> bool {
    match c {
        97...102 => true,
        65...70 => true,
        _ => is_decimal(c)
    }
}

fn is_newline(c: u8) -> bool {
    match c {
        10 => true,
        13 => true,
        _ => false
    }
}

fn is_whitespace(c: u8) -> bool {
    match c {
        9 => true,
        11 => true,
        32 => true,
        _ => false
    }
}

fn is_operator(c: u8) -> bool {
    match c {
        33 => true,
        37 => true,
        38 => true,
        42 => true,
        43 => true,
        45 => true,
        47 => true,
        60 => true,
        61 => true,
        62 => true,
        94 => true,
        124 => true,
        126 => true,
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

        (&TokenType::Directive, &TokenType::LParen) => true,

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

        // Begin
        (&TokenType::Begin, &TokenType::LParen) => true,
        (&TokenType::Begin, &TokenType::Name) => true,
        (&TokenType::Begin, &TokenType::String) => true,
        (&TokenType::Begin, &TokenType::Number) => true,

        // Newline
        (&TokenType::Newline, &TokenType::LParen) => true,
        (&TokenType::Newline, &TokenType::Name) => true,
        (&TokenType::Newline, &TokenType::String) => true,
        (&TokenType::Newline, &TokenType::Number) => true,

        // Everything else
        (_, _) => false

    }
}

