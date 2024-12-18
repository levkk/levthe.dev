#![allow(unused_mut, dead_code, unused_variables)]
use std::iter::{Iterator, Peekable};
use std::ops::{Add, Mul};
use std::collections::HashMap;

/// List of all available tokens in our language.
#[derive(Debug)]
pub enum Token {
    Number(i64),
    Plus,
    String(String),
    Star,
    Identifier(String),
    Let,
    Equals,
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
                '=' => self.tokens.push(Token::Equals),
    
                // All unknown characters are buffered
                // until a known token is seen.
                c => self.buffer.push(c),
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
        
        // If the token is numeric, parse it as a number.
        if let Ok(number) = self.buffer.as_str().parse() {
            self.tokens.push(Token::Number(number));
        } else {
            match self.buffer.as_str() {
                "let" => self.tokens.push(Token::Let),
                // Otherwise, the token is some sort of word,
                // which makes it an identifier.
                _ => self.tokens.push(Token::Identifier(self.buffer.clone())),
            }
        }
        
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

/// Expression term.
#[derive(Debug)]
enum Term {
    /// Constant value.
    Value(Value),

    /// Variable value.
    Variable {
        /// Name of the variable.
        name: String,
    }
}

impl Term {
    /// Evaluate the term given the scope.
    pub fn evaluate(&self, scope: &Scope) -> Value {
        match self {
            Term::Value(value) => value.clone(),
            Term::Variable { name } => {
                match scope.get(name) {
                    Some(value) => value,
                    None => panic!("runtime error: variable '{}' not found", name),
                }
            }
        }
    }
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
            Token::Identifier(name) => Term::Variable { name },
            _ => panic!("syntax error, expected term, got: {:?}", token),
        }
    }
    
    pub fn evaluate(&self, scope: &Scope) -> Value {
        match self {
            // Evaluate a single term.
            Expression::Term(term) => term.evaluate(&scope),

            // Evaluate a binary term.
            Expression::Binary {
                left,
                op,
                right,
            } => {
                // Evaluate the term on the left.
                let left = left.evaluate(scope);

                // Evaluate the term on the right.
                let right = right.evaluate(scope);

                match op {
                    Operation::Addition => {
                        left + right
                    }
                    
                    Operation::Multiplication => {
                        left * right
                    }
                }
            },
        }
    }
}

#[derive(Debug)]
struct Scope {
    variables: HashMap<String, Value>,
}

impl Scope {
    /// Create empty scope.
    pub fn new() -> Scope {
        Scope {
            variables: HashMap::new(),
        }
    }

    /// Retrieve a variable's value from the scope.
    pub fn get(&self, name: &str) -> Option<Value> {
        self.variables.get(name).cloned()
    }

    /// Set a variable's value in the scope.
    pub fn set(&mut self, name: impl ToString, value: Value) {
        self.variables.insert(name.to_string(), value);
    }
}

#[derive(Debug)]
enum Statement {
    Assignment {
        name: String,
        value: Expression,
    },

    Expression(Expression),
}

impl Statement {
    /// Parse a statement from a stream of tokens.
    pub fn parse(stream: &mut Peekable<impl Iterator<Item = Token>>) -> Statement {
        let token = stream.peek().expect("empty token stream");

        match token {
            Token::Let => Self::assignment(stream),
            _ => Statement::Expression(Expression::parse(stream)),
        }
    }

    /// Evaluate the statement given the scope.
    pub fn evaluate(&self, scope: &mut Scope) -> Option<Value> {
        match self {
            Statement::Assignment { name, value } => {
                let value = value.evaluate(scope);
                scope.set(name, value);

                None
            }

            Statement::Expression(expression) => {
                Some(expression.evaluate(scope))
            }
        }
    }

    /// Parse statement assignment.
    fn assignment(stream: &mut impl Iterator<Item = Token>) -> Statement {
        // Consume and discard the `let` token.
        let _let = stream.next().unwrap();

        // Get the variable name.
        let name = match stream.next() {
            None => panic!("syntax error, expected identifier"),
            Some(Token::Identifier(name)) => name,
            Some(token) => panic!(
                "syntax error, expected identifier, got: {:?}",
                token
            ),
        };

        // Consume and discard the `=` token.
        match stream.next() {
            Some(Token::Equals) => (),
            Some(token) => panic!(
                "syntax error, expected '=', got: {:?}",
                token
            ),
            None => panic!("syntax error, expected '='"),
        };

        let value = Expression::parse(stream);

        Statement::Assignment { name, value }
    }
}

fn main () {
    let result = eval("
        let x = 3 * 2
        let y = x + 5
        x + y
    ");

    println!("{:?}", result);
}

fn eval(source: &str) -> Option<Value> {
    let mut scope = Scope::new();
    let mut value = None;

    for line in source.lines() {
        let line = line.trim(); // Remove extra spaces.

        if line.is_empty() {
            continue;
        }

        let mut lexer = Lexer::new(line);
        let tokens = lexer.tokens();

        let statement = Statement::parse(&mut tokens.into_iter().peekable());
        value = statement.evaluate(&mut scope);
    }

    value
}
