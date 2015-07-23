use std::iter;

use compiler::SourceIter;
use parser::Token;
use parser::TokenType;
use parser::Expression;
use parser::BaseLexer;

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

                    if is_expression(self.last_token_type, token_type, self.paren_depth) {

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
                            if is_expression(self.last_token_type, token_type, self.paren_depth) {
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

fn is_expression(last: TokenType, next: TokenType, depth: u8) -> bool {

    match (last, next) {

        // Commas always separate expressions when outside of parenthesis
        (TokenType::Comma, _) if depth == 0 => false,
        (_, TokenType::Comma) if depth == 0 => false,

        // Left Parenthesis
        (TokenType::LParen, TokenType::Name) => true,
        (TokenType::LParen, TokenType::LocalLabelRef) => true,
        (TokenType::LParen, TokenType::Number) => true,
        (TokenType::LParen, TokenType::String) => true,
        (TokenType::LParen, TokenType::Operator) => true,
        (TokenType::LParen, TokenType::LParen) => true,
        (TokenType::LParen, TokenType::RParen) => true,
        (TokenType::LParen, TokenType::MacroArg) => true,

        // Right Parenthesis
        (TokenType::RParen, TokenType::RParen) => true,
        (TokenType::RParen, TokenType::Operator) => true,

        // Operators
        (TokenType::Operator, TokenType::LParen) => true,
        (TokenType::Operator, TokenType::Number) => true,
        (TokenType::Operator, TokenType::String) => true,
        (TokenType::Operator, TokenType::LocalLabelRef) => true,
        (TokenType::Operator, TokenType::Name) => true,
        (TokenType::Operator, TokenType::MacroArg) => true,

        // Numbers
        (TokenType::Number, TokenType::RParen) => true,
        (TokenType::Number, TokenType::Operator) => true,
        (TokenType::Number, TokenType::Comma) => true,

        // Strings
        (TokenType::String, TokenType::RParen) => true,
        (TokenType::String, TokenType::Operator) => true,
        (TokenType::String, TokenType::Comma) => true,

        // LocalLabelRef
        (TokenType::LocalLabelRef, TokenType::RParen) => true,
        (TokenType::LocalLabelRef, TokenType::Operator) => true,
        (TokenType::LocalLabelRef, TokenType::Comma) => true,

        // Names
        (TokenType::Name, TokenType::LParen) => true,
        (TokenType::Name, TokenType::RParen) => true,
        (TokenType::Name, TokenType::Operator) => true,
        (TokenType::Name, TokenType::Comma) => true,

        // Macro Args
        (TokenType::MacroArg, TokenType::LParen) => true,
        (TokenType::MacroArg, TokenType::RParen) => true,
        (TokenType::MacroArg, TokenType::Operator) => true,
        (TokenType::MacroArg, TokenType::Comma) => true,

        // Comma
        (TokenType::Comma, TokenType::LParen) => true,
        (TokenType::Comma, TokenType::Name) => true,
        (TokenType::Comma, TokenType::String) => true,
        (TokenType::Comma, TokenType::Number) => true,
        (TokenType::Comma, TokenType::MacroArg) => true,

        // Directive Values
        (TokenType::Directive, TokenType::LParen) => true,
        (TokenType::Directive, TokenType::Name) => true,
        (TokenType::Directive, TokenType::String) => true,
        (TokenType::Directive, TokenType::Number) => true,
        (TokenType::Directive, TokenType::MacroArg) => true,

        // Instruction Values
        (TokenType::Instruction, TokenType::LParen) => true,
        (TokenType::Instruction, TokenType::Name) => true,
        (TokenType::Instruction, TokenType::String) => true,
        (TokenType::Instruction, TokenType::Number) => true,
        (TokenType::Instruction, TokenType::MacroArg) => true,

        // Everything else
        (_, _) => false

    }
}

