#![allow(unused_mut, dead_code, unused_variables)]
use std::iter::{Peekable, Iterator};

/// List of all available tokens in our language.
#[derive(Debug)]
pub enum Token {
    Number(i64),
    Plus,
}

/// Lexer takes a string and returns a list of tokens.
pub struct Lexer<'a> {
    // Source code.
    source: &'a str,
    // Resulting list of tokens.
    tokens: Vec<Token>,
    // Buffer for multi-character tokens.
    buffer: String,
}

impl<'a> Lexer<'a> {
    /// Create new lexer for the source code.
    pub fn new(source: &'a str) -> Lexer {
        Lexer {
            source,
            tokens: Vec::new(),
            buffer: String::new(),
        }
    }

    /// Convert code into a list of tokens, consuming the lexer.
   pub fn tokens(&mut self) -> Vec<Token> {
        // Extract tokens one character at a time.
        use Token::*;
    
        for c in self.source.chars() {
            match c {
                // Spaces separate tokens.
                ' ' => self.process_token(),
                '0'..='9' => self.buffer.push(c),
                '+' => self.tokens.push(Plus),
                c => todo!("lexer error, unsupported character: '{}'", c),
            }
        }
        
        // Don't forget to process whatever is in the buffer
        // at the end of the input:
        self.process_token();
    
        std::mem::take(&mut self.tokens)
    }
    
    /// Process a multi-character token stored in the buffer.
    fn process_token(&mut self) {
        use Token::*;

        // Empty buffer means no more tokens in the input.
        if self.buffer.is_empty() {
            return;
        }

        // Use the standard library str::parse
        // to convert text to an integer.
        self.tokens.push(
            Number(self.buffer.as_str().parse().unwrap())
        );

        // Clear the buffer for the next token.
        self.buffer.clear();
    }
}

#[derive(Debug)]
enum Operation {
    /// Addition operation.
    Addition,
}

#[derive(Debug)]
enum Value {
    /// A value can be a number. Our language translates
    /// all numbers to 64-bit signed integers, which makes things easier,
    /// but we can add more types later.
    Number(i64),
}

#[derive(Debug)]
enum Term {
    /// A term is just a constant value.
    Value(Value),
}

#[derive(Debug)]
enum Expression {
    /// A single term.
    Term(Term),

    /// A binary operation.
    BinaryOp {
        left: Box<Expression>,
        op: Operation,
        right: Box<Expression>,
    },
}

impl Expression {
    /// Given a stream of tokens, parse a single expression.
    pub fn parse(
        stream: &mut Peekable<impl Iterator<Item = Token>>
    ) -> Expression {
        let left = Self::term(stream);
        let op = stream.next();

        match op {
            Some(op) => {
                let op = match op {
                    Token::Plus => Operation::Addition,
                    _ => panic!("syntax error, expected operator, got: {:?}", op),
                };

                let right = Expression::term(stream);
                Expression::BinaryOp {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                }
            }

            None => left,
        }
    }

    /// Given a stream of tokens, parse a single term.
    fn term(
        stream: &mut Peekable<impl Iterator<Item = Token>>
    ) -> Expression {
        let token = stream.next().expect("parse eof");

        match token {
            Token::Number(n) => {
                Expression::Term(Term::Value(Value::Number(n)))
            }

            _ => panic!("syntax error, expected term, got: {:?}", token),
        }
    }
}

fn main () {
    let source = "21 + 2";
    let mut lexer = Lexer::new(source);
    println!("{:?}", lexer.tokens());
    println!("{:?}", Expression::parse(
        &mut lexer.tokens().into_iter().peekable()
    ));
}