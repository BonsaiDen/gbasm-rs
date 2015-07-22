use compiler::SourceIter;
use parser::Operator;
use parser::Token;

/// Low Level Assembly Tokenizer which only returns uncombined tokens
pub struct BaseLexer<'a> {
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

