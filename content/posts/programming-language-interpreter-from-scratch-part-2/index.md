+++ 
title = 'Programming language interpreter from scratch in Rust: Part 2'
date = 2024-08-08T15:00:00-07:00
+++

In our [last post](/posts/programming-language-interpreter-from-scratch-part-1), we covered foundational topics like the lexer, parser, and interpreting expressions. Let's keep going and learn a few more tricks that will make our programming language more useful.

## Variables

Variables are placeholders for values. Most languages expect them to hold a value at all times, although langauges like C allow them to be null (i.e. empty). Variables can be of different data types, and can be accessible from different levels in the code. Our language currently only allows a single expression, so adding variables will not require much planning.

Let's proceed in the same order as before, starting at the lexer.

### Lexer

A variable is name. When we assign values to variables, e.g. `x = 5`, we are saying that a variable _named_ `x` holds the value `5`. In programming language terms, a name is an identifier, i.e. an entity that refers to something else.

To represent this in the lexer, we need to add another token:

```rust
#[derive(Debug)]
enum Token {
    // Tokens defined in our last post.
    Number(i64),
    Plus,
    String(String),
    Star,

    // The new identifier token.
    Identifier(String),
}
```

When parsing code, an identifier is a token which is not some other token previously defined. For example, when parsing `21 + 2`, all tokens are already spoken for (i.e. `Number, Plus, Number`, spaces are ignored) however, if the code is changed to `21 + x`, the `x` token is not part of our grammar, and  therefore it must be an identity.

Implementing this in the parser requires a two small modifications. First, when parsing source code one at a time, instead of throwing an error when we encounter an unknown character, place it in the buffer:

```rust
match c {
    // ... redacted for brievity
    '*' => self.tokens.push(Token::Star),

    // All unknown characters are buffered
    // until a known token is seen.
    c => self.buffer.push(c),
}
```

Second, instead of treating all buffered tokens as numbers, we need to check if they are in fact identifiers:

```rust
impl<'a> Lexer<'a> {
    fn process_token(&mut self) {
        // Empty buffer means there is nothing to do here.
        if self.buffer.is_empty() {
            return;
        }
        
        // If the token is numeric, parse it as a number.
        if let Ok(number) = self.buffer.as_str().parse() {
            self.tokens.push(Token::Number(number));
        } else {
            // Otherwise, the token is some sort of word,
            // which makes it an identifier.
            self.tokens.push(Token::Identifier(self.buffer.clone()));
        }
        
        self.buffer.clear();
    }
}
```

Testing the lexer against `21 + x`, like so:

```rust
fn main() {
    let source = "21 + x";
    let mut lexer = Lexer::new(source);
    println!("{:?}", lexer.tokens());
}
```

Running our example, we now get:

```
[Number(21), Plus, Identifier("x")]
```

Everything looks good. Naming things is hard, and some languages do not make it easier. For examples, in C/C++, variables cannot start with a number, e.g. variable named `123x` would throw a syntax error. The reasons for this are not entirely clear ([1]), but if we had to take a guess, parsing every identifier is inefficient and slows down compilation. We will not concern ourselves with that for now and allow our language users to name their variables almost whatever they want.


### Parser

sdfs


[1]: https://stackoverflow.com/questions/342152/why-cant-variable-names-start-with-numbers
