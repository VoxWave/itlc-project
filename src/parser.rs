use common::{Direction, Sink, Source, State};
use lexer::{LexError, Token, TokenType, Position};

enum Expression {
    Expressions(Vec<Expression>),
    Lambda(String, Box<Expression>),
    Identifier(String),
}

enum Incomplete {
    Expressions(Vec<Expression>),
    Lambda(String),
}

struct Parser<O>
where
    O: Sink<Result<Expression, ParserError>>,
{
    expression_sink: O,
    parse_stack: Vec<Incomplete>,
}

impl<O> Parser<O> 
where
    O: Sink<Result<Expression, ParserError>>,
{
    fn new(expression_sink: O) -> Parser<O> {
        Parser {
            expression_sink,
            parse_stack: vec![Incomplete::Expressions(Vec::new())],
        }
    }

    fn run<I>(&mut self, mut token_source: I) 
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
    }

    fn print_errors<I>(&mut self, mut token_source: I)
    where 
        I: Source<Result<Token, LexError>>,
    {
        while let Some(r) = token_source.take() {
            match r {
                Err(err) => {
                    println!("{:?}", err);
                },
                _ => {},
            }
        }
        panic!("Lexing failed. Aborting");
    }

    fn normal(&mut self, t: Token) -> State<Parser<O>, Token> {
        match t.token_type {
            TokenType::Bracket(Direction::Left) => {},
            TokenType::Bracket(Direction::Right) => {},
            TokenType::Dot => {
                let msg = "A dot was found without a lambda".to_string();
                let error = ParserError::UnexpectedDot(msg, t.position);
                self.expression_sink.put(Err(error));
            },
            TokenType::Identifier(s) => {},
            TokenType::Lambda => State(Self::lambda),
        }
        State(Self::normal)
    }

    fn lambda(&mut self, t: Token) -> State<Parser<O>, Token> {
        match t.token_type {
            TokenType::Identifier => {},
            _ => {
                let msg = format!("an identifier was expected after a lamda but {:?} was found instead.", t);
                let error = ParserError::InvalidLambdaSyntax(msg, t.position);
                self.expression_sink.push(Err(error));
            },
        }
    }

    fn expect_dot_or_identifier()
}

enum ParserError {
    UnexpectedDot(String, Position),
    InvalidLambdaSyntax(String, Position),
}