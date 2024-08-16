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

A scope is a region of code where a variable is accessible. For example, using Python, let's define a variable inside a function:

```python
def test():
    x = 5

print(x)
```

If you run this code, you will get an error:

```
NameError: name 'x' is not defined
```

The variable `x` is defined in the scope of the function `test` and is not available outside of it.

Our language only has one expression, so rules of scope do not exist yet, but we can already see that we will need a hierarchy of scope objects to retrieve variable values as they are evaluated.

Let's start small and add a scope handler first:

```rust
use std::collections::HashMap;

#[derive(Debug)]
struct Scope {
    variables: HashMap<String, Value>,
}
```

A scope maps variable names to their respective values. For example, if we have the statement `x = 2`, in whichever scope this statement is executed, the variable `x` will be assigned the value `2`.

With a few helper methods, our scope object should be good to go:

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

Now that we have a scope handler, let's add it to the executor.

#### Adding scope

Variables will need to be accessible by all language entities that support them. At this point, variables are only used by terms, so let's encapsulate term execution into its own method. While constant terms are evaluated by just returning the value, variable terms are evaluated by retrieving their current value from the scope:

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



Expressions themselves do not have their own scope, e.g. code like `21 + x` typically uses existing variables and does not define new ones. Therefore, the scope handler is passed into expressions during evaluation as well:


```rust
impl Expression {
    /// Evaluate the expression given the scope.
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


#### Order of evaluation

Evaluating binary expressions can be done in different creative ways. Our interpreter evaluates them left to right. Some languages, e.g. C/C++, do not specify an evaluation order, so each implementation (i.e. each compiler) can do it any way it sees fit ([2]).

For example, in Python, it is not uncommon to write something like this:

```python
if do_thing() or do_another_thing():
    print("We did a thing!")
```

When executed, the interpreter will first run the `do_thing()` function, and if the result is "truthy", e.g. evaluates to `True`, the `do_another_thing()` method is not executed.

If the same code is written in C, either of the functions is liable to be executed, leading to undefined behavior, i.e. a result that is not predictable in advance.

Our interpreter, on the other hand, does neither of those; instead, it evaluates both sides of the binary expression and only then applies the operation to combine them. This is something we, as language authors, get to decide and should explicitly communicate to our users. Not knowing this property can lead to a number of hard to debug problems.

#### Evaluating expressions

Now that we have defined scope and how it is used in our language, let's evaluate our expression:

```rust
fn main() {
    let source = "21 + x";

    // Create a global scope object.
    let mut scope = Scope::new();

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokens();

    // Parse the tokens into an AST.
    let expression = Expression::parse(&mut tokens.into_iter());

    println!("{:#?}", expression);

    // Pass the scope down to the expression.
    let result = expression.evaluate(&scope);

    println!("{:?}", result);
}
```

Running this, we get a new error:

```
thread 'main' panicked at src/main.rs:181:29:
runtime error: variable 'x' not found
```

Great! Our interpreter found the variable, tried to evaluate it, could not find it in the scope, and threw an error.

We do not have support for variable assignment statements yet, so let's hardcode the value of `x` for now:

```rust
scope.set("x", Value::Number(2));
```

Running this, we finally get:

```
Number(23)
```

Finally, let's keep our language definition up-to-date:

```
term = value | variable
variable = identifier
```

Source code is available [here](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=eb59b928b60437430de49a5b7b9c1c94).

### Quick recap

In the lexer, a variable is an identity, which is any kind of space-separated token not previously defined for another purpose. In the parser, variables are another kind of term, which are evaluated given a scope. The scope may contain variable names and values, or it may not, in which case the interpreter should throw an error. Order of evaluation in binary expressions is important, and can vary between programming languages.

## Statements

Statements are instructions for the interpreter that control how, or if, some code is executed. Unlike expressions, statement do not have to evaluate to a value.

For example, a `for` loop instructs the interpreter to execute some code multiple times, while automatically assigning a different value to a variable. The loop stops when a certain condition is met, e.g. the variable reaches a certain value in a sequence.

Some statements are more complex than others, and we will start our implementation with the simplest one: variable assignment.

### Variable assignment

Variable assignment is a statement which places a value into a variable. This variable can then be reused in later parts of the code inside other statements or expressions.

For example, in Rust, variable assignment can be as simple as:

```rust
let x = 5;
```

where the variable `x` is assigned the value `5`. Slightly more complex examples assign the value of an expression to a variable:

```rust
let x = 5 + 2;
```

#### Formal definition

Choosing the right syntax for variable assignment is a mix of taste and experience. Ultimately, we want to make it clear that a statement is, in fact, performing variable assignment, and not something else, like mutating another variable.

In Rust, variable assignment takes a few forms, but we can get inspired from the most common one  with this formal definition:

```
statement = "let" identifier "=" expression
```

Let's unpack this. A statement starts with the keyword `let`. This keyword, by itself, only serves to indicate that this statement is a variable assignment. What follows this keyword is an identifier, which as we learned earlier, is any token which is not some other known token, and is used to name variables. Following the identifier, we see an equals sign. By itself, just like the `let` keyword, the `=` token does not do anything except to tell us what comes next, which is an expression.

You will note that we do not use semicolons at the end of statements. This is a design choice. Some languages, like JavaScript, make semicolons optional. Others, like C/C++, require them.

Let's implement this, starting at, like before, the lexer.

### Lexer

Variable assignment introduces two tokens we have not seen before: `let` and `=`. Let's add them to our enum:

```rust
#[derive(Debug)]
pub enum Token {
    Number(i64),
    Plus,
    String(String),
    Star,
    Identifier(String),

    /// `let` token.
    Let,

    /// `=` token.
    Equals,
}
```

Parsing the single-character equals token can be done just like all the others:

```rust
while let Some(c) = chars.next() {
    // ... redacted for brevity

    // Handle the equals token.
    '=' => self.tokens.push(Token::Equals),

    // Buffer everything else.
    c => self.buffer.push(c),
}
```

The `let` token contains multiple characters, so we will handle it in the `process_token` method:

```rust
// If the token is numeric, parse it as a number.
match self.buffer.as_str() {
    // Handle the "let" keyword.
    "let" => self.tokens.push(Token::Let),

    // Otherwise, the token is some sort of word,
    // which makes it an identifier.
    _ => self.tokens.push(Token::Identifier(self.buffer.clone())),
}
```

Instead of assuming all words are identifiers, we are now checking them against a list of special words. This list is called reserved keywords: words which are part of the language itself and cannot be used for naming variables, functions or other user-created objects.

As our language evolves, we will continue to remove words from the `Identifier` token, and add them to the reserved keywords list. In Rust, for example, keywords like `if`, `for`, `where` and many more are reserved and cannot be used to name variables, functions or structs, without escaping them, which in my opinion, adds a lot of confusion.

With the lexer handled, let's move on to the parser.

### Parser

Statements are new new entities, and just like `Token` and `Expression`, need their own enum:

```rust
#[derive(Debug)]
enum Statement {
    /// Variable assignment statement.
    Assignment {
        /// Name of the variable.
        name: String,

        /// Value of the variable, dynamically evaluated.
        value: Expression,
    },
}
```

Variable assignment, contains two pieces of information: the name of the variable and the expression which, when evaluated, produces the value stored in the variable.

You will note that the expression is stored as-is, i.e. it is not evaluated (converted to a value). This is intentional: the expression can contain other variables or even functions that we have not defined yet, so evaluating it now could produce an error.

Moving on, just like with `Expression`, let's implement parsing statements from a stream of tokens:

```rust
use std::iter::Peekable;

impl Statement {
    /// Parse a statement from a stream of tokens.
    pub fn parse(stream: &mut Peekable<impl Iterator<Item = Token>>) -> Statement {
        let token = stream.peek().expect("empty token stream");

        match token {
            Token::Let => Self::assignment(stream),
            _ => panic!("syntax error, expected 'let', got: {:?}", token),
        }
    }
}
```

We expect to add more statements later, so the logic here is simple: if we see the `let` token, it is an assignment, and we parse it another method, defined below:

```rust
impl Statement {
    /// Parse a variable assignment statement.
    fn assignment(stream: &mut impl Iterator<Item = Token>) -> Statement {
        // Consume and discard the `let` token.
        // We peeked it above, but have not consumed it yet.
        let _let = stream.next().unwrap();

        // Get the variable name. The name has to be an identifier
        // token; anything else produces a syntax error.
        let name = match stream.next() {
            None => panic!("syntax error, expected identifier"),
            Some(Token::Identifier(name)) => name,
            Some(token) => panic!(
                "syntax error, expected identifier, got: {:?}",
                token
            ),
        };

        // Consume and discard the `=` token.
        // If the token is not there, or it's something else,
        // throw a syntax error.
        match stream.next() {
            Some(Token::Equals) => (),
            Some(token) => panic!(
                "syntax error, expected '=', got: {:?}",
                token
            ),
            None => panic!("syntax error, expected '='"),
        };

        /// Parse the expression.
        let value = Expression::parse(stream);

        Statement::Assignment { name, value }
    }
}
```

The `let` and `=` tokens are discarded, while the `Identifier` and the `Expression` are parsed and stored for later evaluation.

Quick note on the `Peekable` iterator. `Peekable` allows the user to look ahead without consuming tokens from the stream. This is useful for parsers that need to backtrack, i.e. go back, when they see some tokens that are not part of some statement syntax. Right now, our syntax is very simple: we only allow a single statement type. As we add more statements, our language will become more ambiguous, and a sequence of tokens will not clearly identify a particular statement until we read at least two or more tokens off the stream.

With the parser finished, let's execute our first statement.

### Executor

The assignment statement is actually pretty easy to implement:

```rust
impl Statement {
    /// Evaluate a statement, optionally returning a value.
    pub fn evaluate(&self, scope: &mut Scope) -> Option<Value> {
        match self {
            Statement::Assignment { name, value } => {
                let value = value.evaluate(scope);
                scope.set(name, value);

                None
            }
        }
    }
}
```

All we are doing here is evaluating the expression and storing the variable name and the value in the scope. Since this operation does not evaluate to a value (it could, but we chose not to), the return result is `None`, i.e. this statement produces no return value.

Let's test what we have so far:

```rust
fn main() {
    let mut scope = Scope::new();
    let mut lexer = Lexer::new("let x = 5 + 2");

    let tokens = lexer.tokens();

    let statement = Statement::parse(&mut tokens.into_iter().peekable());
    println!("{:#?}", statement);

    statement.evaluate(&mut scope);

    println!("{:?}", scope.get("x"));
}
```

which produces:

```
Assignment {
    name: "x",
    value: Binary {
        left: Value(
            Number(
                5,
            ),
        ),
        op: Addition,
        right: Value(
            Number(
                2,
            ),
        ),
    },
}

Some(Number(7))
```

Looking good! We have support for a statement which can create and assign values to variables. Our language is becoming more useful. Code is available [here](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=3917b490136c9b2e9e33feacd68de8d7).

## Running real code

By themselves, statements and expressions are nice, but combining them together produces an actual programming language. Let's parse and execute the following piece of code:

```rust
let x = 3 * 2
let y = x + 5
x + y
```

If you recall the formal definition of our language from our [last post](/posts/programming-language-interpreter-from-scratch-part-1), our language does not allow multiple statements; in fact, our language is currently defined as just one expression:

```
language = expression
```

Let's give it a quick update:

```
language = [statement]
statement = "let" identifier "=" expression | expression
```

A program written in our language is now a list of statements. Any of the statements in that list can be a variable assignment or an expression.

Since we have not added any new tokens, we do not need to modify the lexer, and can jump straight into the parser:

```rust
#[derive(Debug)]
enum Statement {
    /// Variable assignment.
    Assignment {
        name: String,
        value: Expression,
    },

    /// An expression.
    Expression(Expression),
}
```

Any statement which is not an assignment is then by definition an expression:

```rust
impl Statement {
    /// Parse the statement.
    pub fn parse(stream: &mut Peekable<impl Iterator<Item = Token>>) -> Statement {
        let token = stream.peek().expect("empty token stream");

        match token {
            /// Let token indicates an assignment.
            Token::Let => Self::assignment(stream),

            /// Any other token indicates an expression.
            _ => Statement::Expression(Expression::parse(stream)),
        }
    }
}
```

If a statement contains a single expression, executing that statement in the interpreter will return the result of that expression:

```rust
impl Statement {
    /// Evaluate the statement given the scope.
    pub fn evaluate(&self, scope: &mut Scope) -> Option<Value> {
        match self {
            /// Evaluate the expression to a single value.
            Statement::Expression(expression) => Some(expression.evaluate(scope)),

            /// Save variable in the scope.
            Statement::Assignment { name, value } => {
                let value = value.evaluate(scope);
                scope.set(name, value);

                None
            }
        }
    }
}
```

#### `eval`

If you are familiar with interpreted languages like Python or Ruby, you know that both of them have a similar method to dynamically evaluate code. For example, if you open up the Ruby interpreter (`irb`) and run this:

```ruby
eval("x = 5 + 2")
```

you will get:

```
=> 7
```

This is useful for something called "metaprogramming". Metaprogramming ([3]) is a technique for writing and executing new code during runtime: when a program is executed, it writes and executes more code.

While a powerful feature, for us, implementing this function serves a simpler purpose: executing multiple lines of our language:

```rust
/// Parse and execute code written in our language.
fn eval(source: &str) -> Option<Value> {
    let mut scope = Scope::new();
    let mut value = None;

    for line in source.lines() {
        // Remove trailing/leading spaces/new line character.
        // Our lexer is not very resilient at the moment.
        let line = line.trim();

        // Do not execute empty lines of code.
        if line.is_empty() {
            continue;
        }

        // Read, parse, and execute the line of code.
        let mut lexer = Lexer::new(line);
        let tokens = lexer.tokens();

        let statement = Statement::parse(&mut tokens.into_iter().peekable());

        // Save the last value only.
        value = statement.evaluate(&mut scope);
    }

    value
}
```



Running our example is now easy:

```rust
fn main () {
    let result = eval("
        let x = 3 * 2
        let y = x + 5
        x + y
    ");

    println!("{:?}", result);
}
```

which produces:

```
Some(Number(17))
```

Full code is available [here](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=f9eeaef8b32b8de453ec536a8654d8df).


[1]: https://stackoverflow.com/questions/342152/why-cant-variable-names-start-with-numbers
[2]: https://en.cppreference.com/w/c/language/eval_order
[3]: https://en.wikipedia.org/wiki/Metaprogramming
