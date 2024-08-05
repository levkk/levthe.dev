+++
title = 'Build your own language interpreter from scratch'
date = 2024-08-01T13:33:45-04:00
+++

Creating your own programming language is not hard. In fact, once you understand the basic building blocks, it can even be fun.

All programming languages, from the simplest one like Python to the incomprehensible one like C++, are built using three independent components: a lexer, a parser, and an translator.

In this post, we will cover all three of them, implement them in the Internet's favorite language du jour, Rust, and run a program written in our own language.

All code examples will also have a Rust Playground link, so you can them out as you follow along.

Without further ado, let's get started.

## Lexer

The lexer takes text and converts it to a list of tokens. A token could be a letter, a word, a number, or a punctuation mark. Tokens have meaning depending on where they are situated in the code, but we will not concern ourselves with that just yet.

Let's start with a simple program that goes something like this:

```
1 + 2
```

The lexer's job is to read this code and produce a sequence of tokens, as follows:

```
Number(1), Plus, Number(2)
```


While both you and I strongly suspect that this program will add one and two together to produce `3`, the lexer will not make any assumptions at this stage.

Let's jump into some code. First, let's define the list of tokens our language will support:

```rust
/// List of all available tokens in our language.
#[derive(Debug)]
pub enum Token {
    Number(i64),
    Plus,
}
```

Keep this list nearby, we'll be adding more tokens to it later as our language evolves. It can only add numbers at the moment, but we'll be adding more features later like control flow with if-statements and for-loops.

### Extracting tokens

At the very basic level, a token is a single character, like the plus sign (`+`). Our lexer (like all the others) will therefore process the source code one character at a time.

```rust
/// Lexer takes a string and returns a list of tokens.
pub struct Lexer<'a> {
    // Source code.
    source: &'a str,
    // Resulting list of tokens.
    tokens: Vec<Token>,
}

impl<'a> Lexer<'a> {
    /// Create new lexer for the source code.
    pub fn new(source: &'a str) -> Lexer {
        Lexer {
            source,
            tokens: Vec::new(),
        }
    }

    /// Convert code into a list of tokens, consuming the lexer.
    pub fn tokens(mut self) -> Vec<Token> {

        // Extract tokens one character at a time.
        for c in self.source.chars() {
            todo!()
        }

        self.tokens
    }
}
```

Our language only supports adding numbers, so our tokens can range between `0` and `9`, and `+`:

```rust
pub fn tokens(mut self) -> Vec<Token> {
    use Token::*;

    // Extract tokens one character at a time.
    for c in self.source.chars() {
        match c {
            '0'..='9' => self.tokens.push(
                Number(c.to_digit(10).unwrap() as i64)
            ),
            '+' => self.tokens.push(Plus),
            c => todo!("lexer error, unsupported character: '{}'", c),
        }
    }

    self.tokens
}
```

While simple, this code does a lot. We extract characters from text, interpret their meaning in the context of our code, and produce values that our compiler can understand. The separation of concerns between the lexer and the parser makes building a compiler a lot easier.

Before writing any more code, we should probably write a simple test to make sure we're on the right track. I'm using Rust Playground so we can just write a `main` function:

```rust
fn main () {
    let source = "1 + 2";
    let lexer = Lexer::new(source);
    println!("{:?}", lexer.tokens());
}
```

Everything seems to be in order, let's run our code:

```
thread 'main' panicked at src/main.rs:38:22:
not yet implemented: lexer error, unsupported character: ' '
```

If you have done this before, you probably noticed this beforehand, but for all the beginners this may come as a surprise: the space character (` `) is also a token which is part of the language syntax. Therefore, the lexer needs to handle it. We do not have a use for the space token yet, beyond separating other tokens, so we will simply ignore it:

```rust
match c {
    // Ignore spaces.
    ' ' => continue,
    '0'..='9' => self.tokens.push(
        Number(c.to_digit(10).unwrap() as i64)
    ),
    '+' => self.tokens.push(Plus),
    c => todo!("lexer error, unsupported character: '{}'", c),
}
```

Running our lexer now with support for spaces, we are getting the right result:

```
[Number(1), Plus, Number(2)]
```

As the designer of your own language, you get to decide what goes and what breaks your code. Some languages for example would throw a syntax error if a space was missing (e.g. Python, which uses it for indentation).

### Multi-character tokens
We are keeping our language simple on purpose so we can cover the whole process without getting overwhelmed, but before we go to writing the parser, let's add support for multi-character numbers. After all, our language would be pretty silly if it could only count to 9.

Since our lexer reads the code one character at a time and numbers can effectively be infinitely large, we need to add a buffer to collect digits as we consume them from the input:

```rust
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
    /// Process a multi-character token stored in the buffer.
    fn process_token(&mut self) {
        use Token::*;

        // Empty buffer means no more tokens in the input.
        if self.buffer.is_empty() {
            return;
        }

        // Use the standard library's [`str::parse`]
        // to convert text to an integer.
        self.tokens.push(
            Number(self.buffer.as_str().parse().unwrap())
        );

        // Clear the buffer for the next token.
        self.buffer.clear();
    }
}
```

Now that we handle multi-character numbers, the space token needs to be handled differently. Instead of just ignoring it, we will process whatever is in the buffer instead:

```rust
for c in self.source.chars() {
    match c {
        // Spaces separate tokens.
        ' ' => self.process_token(),
        // Buffer number characters
        // instead of parsing them individually.
        '0'..='9' => self.buffer.push(c),
        '+' => self.tokens.push(Plus),
        c => todo!("lexer error, unsupported character: '{}'", c),
    }
}

// Don't forget to process whatever is in the buffer
// at the end of the input:
self.process_token();
```

Let's change the source code of our program and try the lexer now with actual support for numbers:

```rust
let source = "21 + 2";
```

Interpreting the new code produces the expected stream of tokens:

```
[Number(21), Plus, Number(2)]
```

It may seem like our lexer is still incomplete since we haven't tried adding support for more advanced control flow like `if` or `for` instructions, but actually, since it supports parsing multi-character tokens, adding more tokens will be easy. More on that later though, let's move forward with our compiler and build its second component: the parser.

The full source code for the lexer is available on [Rust Playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=8f1a2f1b32394bafbcc0a3f5f8a53159).

## Parser

The parser takes a list (also called a stream) of tokens and produces an Abstract Syntax Tree. The AST is a structural representation of code: it organizes tokens in such a way that makes sense in the context of the language rules. The AST validates that the code is syntactically correct.

Before we can build our parser, we need to get a bit more involved in our language design. A parser needs us to have a formal definition of what our language can do. Our example is adding numbers, so let's start with that and add more features as we go.

### Formal definition

A formal definition for our language serves as a blueprint of what our language can do. Our language is pretty simple, so the definition could go something like this:

```
language = expression
expression = term operation term | term
term = value
value = number
operation = '+'
```

Before trying to understand what this means, let's throw in some formal definitions:

| Term | Definition |
|------|------------|
| Language | The programming language we are designing. |
| Expression | A piece of code which when executed produces a single value as a result. Expressions are composed of terms and operations. |
| Term | A value, either constant or variable. |
| Operation | An action that combines two expressions into one. |

Armed with these, let's go through our language definition line by line.
W
```
language = expression
```

Our entire language is just one expression. Programs written in this language effectively can contain only one line of code. A bit boring, but we will expand this later quite easily.

```
expression = term operation term | term
```

An expression in our language is either a binary operation, or a single expression. While somewhat confusing at first, this recursive definition is simpler than it appears: it means an expression can be either composed of two expressions joined together by an operation, e.g. addition, or just be an expression by itself, like a constant or a variable. Going down to the next line, things are starting to make sense:

```
expression = term
```

An expression can just be a term. Since a term can be a constant, an expression can simply be a number, like `12`, or an operation on two terms, like our original example: `21 + 2`.

The next 3 lines complete our formal definition:

```
term = value
value = number
operation = '+'
```

A term can be a value, a value is a number, and an operation can be an addition. Any line of code we write in our language can be represented using this formal definition. So, to make a parser for our language, we just need to implement this definition.

### Basic blocks

Starting from the bottom, let's implement the operation, value and term:

```rust
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
```

### Expression

Now that we have our basic building blocks, let's define an expression:

```rust
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
```

Just like the formal definition, an expression can either be a term or a binary operation on two expressions. In Rust, recursive enum definitions require the use of a `Box` to make sure the type has a finite size on the stack; the `Box` is a fixed-size pointer to an object on the heap.

That's not a lot of code given our long theoretical explanation, but that's kind of the point: once we understand the theory, implementing the parser becomes straight forward.

### Process tokens

Just like the lexer, the parser processes tokens one at a time. Since our language can only have one expression for now, we can implement the parser directly on the expression enum:

```rust
use std::iter::{Peekable, Iterator};

impl Expression {
    /// Given a stream of tokens, parse a single expression.
    pub fn parse(
        stream: &mut Peekable<impl Iterator<Item = Token>>
    ) -> Expression {
        todo!()
    }

    /// Given a stream of tokens, parse a single term.
    fn term(
        stream: &mut Peekable<impl Iterator<Item = Token>>
    ) -> Expression {
        todo!()
    }
}
```