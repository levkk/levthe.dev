+++ 
title = 'Programming language interpreter from scratch in Rust: Part 2'
date = 2024-08-08T15:00:00-07:00
draft = true
+++

In our [last post](/posts/programming-language-interpreter-from-scratch-part-1), we covered foundational topics like the lexer, parser, and interpreting expressions. Let's keep going and learn a few more tricks that will make our programming language more useful.

## Variables

Variables are placeholders for values. Most languages expect them to hold a value at all times, although langauges like C allow them to be null (i.e. empty). Variables can be of different data types, and can be accessible from different levels in the code. Our language currently only allows a single expression, so adding variables will not require much planning.

Let's proceed in the same order as before, starting at the lexer.

### Lexer

In the lexer, a variable is an identifier. When we assign values to variables, e.g. `x = 5`, we are stating that a variable named `x` holds the value `5`. In programming language terms, an identifier is a name of some object, i.e. an entity that refers to something else.

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

When reading code, an identifier is a token which is not some other token previously defined. For example, when parsing `21 + 2`, all tokens are already spoken for (i.e. `Number, Plus, Number`, spaces are ignored) however, if the code is changed to `21 + x`, the `x` token is not part of our grammar, and  therefore it must be an identity.

Implementing this in the parser requires a two small modifications. First, when parsing source code one at a time, instead of throwing an error when we encounter an unknown character, place it in the buffer:

```rust
match c {
    // ... redacted for brevity
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

Let's test the lexer with `21 + x`, like so:

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

The parser now needs to handle a new token, the identifier. Our language only supports a single expression at the moment, so even though variables can be used in a few different contexts, for us, we only need to add it to the expression.

A variable in an expression is another type of term. In our example, `21 + x`, the constant term on the left is added to a variable term on the right. Adding another kind of term is just another enum variant:

```rust
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
```

Having added the term type, let's add it to the parser:

```rust
impl Expression {
    /// Given a stream of tokens, parse a single term.
    fn term(stream: &mut impl Iterator<Item = Token>) -> Term {
        let token = stream.next().expect("expected a token");

        match token {
            // Constant number value.
            Token::Number(n) => Term::Value(Value::Number(n)),
            // Constant string value.
            Token::String(s) => Term::Value(Value::String(s)),
            // An identifier. Since our identifiers are currently only variables,
            // we can safely convert this token to a variable term
            // with the given name.
            Token::Identifier(name) => Term::Variable { name },
            _ => panic!("syntax error, expected term, got: {:?}", token),
        }
    }
}
```

Before we compile this, let's add a `todo!` into the executor `evaluate` method; we do not know how to evaluate variable terms yet:

```rust
impl Expression {
    pub fn evaluate(&self) -> Value {
        match self {
            // Single term expression holding a value.
            Expression::Term(Term::Value(value)) => value.clone(),
            // Binary expression evaluating two values.
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
            // Any kind of variable term expressions.
            // Currently unhandled.
            _ => todo!("evaluate variable expressions"),
        }
    }
}
```

Evaluating our expression, `21 + x`, will now produce:

```
Binary {
    left: Value(
        Number(
            21,
        ),
    ),
    op: Addition,
    right: Variable {
        name: "x",
    },
}
thread 'main' panicked at src/main.rs:238:18:
not yet implemented: evaluate variable expressions
```

Looks good. Our parser can understand variable terms; it does not know how to execute them yet, and before we implement that, let's do a short detour and talk about variable scopes.

### Scope

A scope is a region of code where a variable is accessible. For example, if you write something like this and run it in a Python interpreter,

```python
def test():
    x = 5

print(x)
```

you will get an error:

```
NameError: name 'x' is not defined
```

The variable `x` is defined in the scope of the function `test` and is not available anywhere else.

Our language only has one expression, so rules of scope do not exist yet, but we can already see that we will need a hierarchy of scope objects to retrieve variable values as they are evaluated.

Let's start simple and add a scope handler to our interpreter:

```rust
use std::collections::HashMap;

#[derive(Debug)]
struct Scope {
    variables: HashMap<String, Value>,
}
```

A scope maps variable names to their respective values. For example, if we have the statement `x = 2`, in whichever scope this statement is executed, the variable `x` will be assigned the value 2.

Let's add a couple helper methods:

```rust
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
```

Now that we have defined a variable scope, let's add it to our executor.

### Adding scope

```rust
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
```


```rust
impl Expression {
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
```

```rust
fn main() {
    let source = "21 + x";
    let mut scope = Scope::new();
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokens();

    // Parse the tokens into an AST.
    let expression = Expression::parse(&mut tokens.into_iter());

    println!("{:#?}", expression);

    // Execute the AST producing a single value.
    let result = expression.evaluate(&scope);

    println!("{:?}", result);
}
```

```
thread 'main' panicked at src/main.rs:181:29:
runtime error: variable 'x' not found
```

```rust
scope.set("x", Value::Number(2));
```

```
Number(23)
```

Source code is available [here](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=eb59b928b60437430de49a5b7b9c1c94).

[1]: https://stackoverflow.com/questions/342152/why-cant-variable-names-start-with-numbers
