use common::{Sink, Source};
use lexer::{LexError, Token};

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
    buffer: Vec<Token>,
    //parse_stack: Vec<(Incomplete, Vec<Token>)>,
}

impl<O> Parser<O> 
where
    O: Sink<Result<Expression, ParserError>>,
{
    fn new() -> Self {

    }

    fn run<I>(&mut self, token_source: I) 
    where
        I: Source<Result<Token, LexError>>,
    {
        let mut state = State(Self::normal);
        let mut error = false;
        while let Some(r) = token_source.take() {
            match r {
                Ok(t) => {

                }
                Err(err) => {
                    println!("{}", err);
                    error = true;
                    break;
                }
            }
            state = state(self, t);
        }
        if error {
            self.print_errors(token_source);
        }
    }

    fn print_errors(&mut self, token_source: I)
    where 
        I: Source<Result<Token, LexError>>,
    {
        while let Some(r) = token_source.take() {
            Err(err) => {
                println!("{}", err);
            },
            _ => {},
        }
    }

    fn normal(&mut self, t: Token)
}

enum ParserError {

}