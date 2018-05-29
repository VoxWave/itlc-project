use common::{Direction, Sink, Source, State};
use lexer::{LexError, Position, Token, TokenType};

enum Expression {
    Application(Vec<Expression>),
    Lambda(String, Box<Expression>),
    Variable(String),
}

enum Incomplete {
    Expressions(Vec<Expression>),
    Lambda(String, Vec<Expression>),
}

struct Parser
{
    parse_stack: Vec<Incomplete>,
}

impl Parser
{
    fn new() -> Parser {
        Parser {
            parse_stack: vec![Incomplete::Expressions(Vec::new())],
        }
    }

    fn run<I>(&mut self, mut token_source: I) -> Expression
    where
        I: Source<Result<Token, LexError>>,
    {
        let mut state = State(Self::normal);
        let mut error = false;
        while let Some(r) = token_source.take() {
            match r {
                Ok(t) => {
                    state = state(self, t);
                }
                Err(err) => {
                    println!("{:?}", err);
                    error = true;
                    break;
                }
            }
        }
        if error {
            self.print_errors(token_source);
        }
        self.construct_expression()
    }

    fn construct_expression(&mut self) -> Expression {
        Expression::Variable(String::from("placeholder"))
    }

    fn print_errors<I>(&mut self, mut token_source: I)
    where
        I: Source<Result<Token, LexError>>,
    {
        while let Some(r) = token_source.take() {
            match r {
                Err(err) => {
                    println!("{:?}", err);
                }
                _ => {}
            }
        }
        panic!("Lexing failed. Aborting");
    }

    fn normal(&mut self, t: Token) -> State<Parser, Token> {
        match t.token_type {
            TokenType::Bracket(Direction::Left) => {
                let incomplete = Incomplete::Expressions(Vec::new());
                self.parse_stack.push(incomplete);
                State(Self::normal)
            }
            TokenType::Bracket(Direction::Right) => {
                if self.parse_stack.len() > 1 {
                    let incomplete = self.parse_stack.pop().unwrap();
                    match incomplete {
                        Incomplete::Expressions(v) => {
                            let expression = if v.len() > 1 {
                                Expression::Application(v)
                            } else if let Some(e) = v.pop() {
                                e
                            } else {
                                return State(Self::normal);
                            };
                            match self.parse_stack.pop().unwrap() {
                                Incomplete::Expressions(e) => e.push()
                                Incomplete::Lambda(_, e) => {},
                            }
                        },
                        Incomplete::Lambda(i, v) => {},
                    }
                } else {
                    println!("Unexpected closing bracket at {:?}", t.position);
                    panic!();
                }
            }
            TokenType::Dot => {
                println!("A dot was found without a lambda at {:?}", t.position);
                panic!();
            }
            TokenType::Identifier(s) => {
                let expression = Expression::Variable(s);
                match self.parse_stack.pop() {
                    Some(incomplete) => {
                        match incomplete {
                            Incomplete::Expressions(v) => v.push(expression),
                            Incomplete::Lambda(_, v) => v.push(expression),
                        }
                        self.parse_stack.push(incomplete);
                    },
                    None => unreachable!(),
                }
                State(Self::normal)
            },
            TokenType::Lambda => State(Self::lambda),
        }
    }

    fn lambda(&mut self, t: Token) -> State<Parser, Token> {
        match t.token_type {
            TokenType::Identifier(s) => {
                self.parse_stack.push(Incomplete::Lambda(s, Vec::new()));
                State(Self::expect_dot_or_identifier)
            }
            _ => {
                println!("an identifier was expected after a lamda");
                println!("found {:?} instead.", t);
                panic!();
            }
        }
    }

    fn expect_dot_or_identifier(&mut self, t: Token) -> State<Parser, Token> {
        match t.token_type {
            TokenType::Dot => State(Self::normal),
            TokenType::Identifier(s) => {
                self.parse_stack.push(Incomplete::Lambda(s, Vec::new()));
                State(Self::expect_dot_or_identifier)
            }
            _ => {
                println!("a dot or an identifier was expected after a lambda and an identifier");
                println!("found {:?} instead.", t);
                panic!();
            }
        }
    }
}
