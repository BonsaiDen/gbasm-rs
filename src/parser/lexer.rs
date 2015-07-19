use parser::Operator;
use parser::Token;
use compiler::SourceIter;

pub struct Lexer<'a> {
    source: &'a mut SourceIter,
    token: Token
}

impl <'a>Lexer<'a> {

    pub fn new(source: &'a mut SourceIter) -> Lexer<'a> {

        // Goto first byte in iterator
        source.next();

        Lexer {
            source: source,
            token: Token::Eof
        }

    }

    pub fn get<'b>(&'b self) -> &'b Token {
        &self.token
    }

    pub fn next<'b>(&'b mut self) -> &'b Token {

        let ch: u8 = self.source.get();
        let next: u8 = self.source.peek();

        self.token = match ch {

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

        };

        &self.token

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

