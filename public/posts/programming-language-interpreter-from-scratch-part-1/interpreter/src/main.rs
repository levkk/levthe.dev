#![allow(unused_mut, dead_code, unused_variables)]
use std::iter::Iterator;
use std::ops::{Add, Mul};

/// List of all available tokens in our language.
#[derive(Debug)]
pub enum Token {
    Number(i64),
    Plus,
    String(String),
    Star,
}

pub struct Lexer<'a> {
    // Source code.
    source: &'a str,
    // Resulting list of tokens.
    tokens: Vec<Token>,
    // Buffer for multi-character tokens.
    buffer: String,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Lexer {
        Lexer {
            source,
            tokens: Vec::new(),
            buffer: String::new(),
        }
    }

    /// Extract tokens one character at a time.
    pub fn tokens(&mut self) -> Vec<Token> {
        let mut chars = self.source.chars();
    
        while let Some(c) = chars.next() {
            match c {
                ' ' => self.process_token(),
                '0'..='9' => self.buffer.push(c),
                '+' => self.tokens.push(Token::Plus),
    
                // Double quote indicating the start of a string.
                '"' => {
                    let mut string = String::new();
    
                    while let Some(c) = chars.next() {
                        match c {
                            // Closing double quote ends the string.
                            '"' => break,
                            _ => string.push(c),
                        }
                    }
    
                    self.tokens.push(Token::String(string));
                },
                '*' => self.tokens.push(Token::Star),
    
                c => todo!("lexer error, unsupported character: '{}'", c),
            }
        }
    
        self.process_token();
    
        std::mem::take(&mut self.tokens)
    }

    fn process_token(&mut self) {
        // Empty buffer means there is nothing to do here.
        if self.buffer.is_empty() {
            return;
        }

        // Use the standard library's [`str::parse`]
        // to convert text to an integer.
        self.tokens
            .push(Token::Number(self.buffer.as_str().parse().unwrap()));

        // Clear the buffer for the next token.
        self.buffer.clear();
    }
}

/// An operation. Only addition currently supported.
#[derive(Debug)]
enum Operation {
    Addition,
    Multiplication,
}

/// A constant value. Currently, only numbers are supported.
#[derive(Debug, Clone)]
enum Value {
    Number(i64),
    /// A value storing a string.
    String(String),
}

impl Add for Value {
    type Output = Value;

    fn add(self, other: Value) -> Value {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a + b),

            // Supports 21 + "hello world"
            (
                Value::Number(a),
                Value::String(s),
            ) => Value::String(a.to_string() + &s),

            // Supports "hello world" + 21
            (
                Value::String(s),
                Value::Number(a),
            ) => Value::String(s + a.to_string().as_str()),

            (a, b) => todo!(
                "syntax error, '+' between {:?} and {:?} not supported",
                a, b
            ),
        }
    }
}

impl Mul for Value {
    type Output = Value;

    fn mul(self, other: Value) -> Value {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a * b),

            // Supports 21 + "hello world"
            (
                Value::Number(a),
                Value::String(s),
            ) => Value::String(s.repeat(a as usize)),

            // Supports "hello world" + 21
            (
                Value::String(s),
                Value::Number(a),
            ) => Value::String(s.repeat(a as usize)),

            (a, b) => todo!(
                "syntax error, '+' between {:?} and {:?} not supported",
                a, b
            ),
        }
    }
}

/// Expression term. Currently, only values are supported.
#[derive(Debug)]
enum Term {
    Value(Value),
}

#[derive(Debug)]
enum Expression {
    /// A binary operation.
    Binary {
        left: Term,
        op: Operation,
        right: Term,
    },

    /// Just a term by itself.
    Term(Term),
}

impl Expression {
    /// Given a stream of tokens, parse a single expression.
    pub fn parse(stream: &mut impl Iterator<Item = Token>) -> Expression {
        let left = Self::term(stream);
        let operation = stream.next();

        match operation {
            Some(operation) => {
                let op = match operation {
                    Token::Plus => Operation::Addition,
                    Token::Star => Operation::Multiplication,
                    _ => panic!("syntax error, expected operation, got: {:?}", operation),
                };

                let right = Self::term(stream);

                Expression::Binary { left, op, right }
            }

            None => Expression::Term(left),
        }
    }

    /// Given a stream of tokens, parse a single term.
    fn term(stream: &mut impl Iterator<Item = Token>) -> Term {
        let token = stream.next().expect("expected a token");

        match token {
            Token::Number(n) => Term::Value(Value::Number(n)),
            Token::String(s) => Term::Value(Value::String(s)),
            _ => panic!("syntax error, expected term, got: {:?}", token),
        }
    }
    
    pub fn evaluate(&self) -> Value {
        match self {
            Expression::Term(Term::Value(value)) => value.clone(),
            Expression::Binary {
                left: Term::Value(left),
                op,
                right: Term::Value(right)
            } => {
                match op {
                    Operation::Addition => {
                        left.clone() + right.clone()
                    }
                    
                    Operation::Multiplication => {
                        left.clone() * right.clone()
                    }
                }
            },
        }
    }
}

fn main () {
    eval(r#""hello" * 3"#);
}

fn eval(source: &str) {
    // Parse the code into tokens.
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokens();

    // Parse the tokens into an AST.
    let expression = Expression::parse(&mut tokens.into_iter());

    println!("{:#?}", expression);

    // Execute the AST producing a single value.
    let result = expression.evaluate();

    println!("{:?}", result);
}
