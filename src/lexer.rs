use common::{Direction, Sink, Source};

pub struct Token {
    token_type: TokenType,
    position: Position,
}

pub struct Point {
    pub row: usize,
    pub column: usize,
}

pub struct Position {
    starting_point: Point,
    ending_point: Point,
}

pub enum TokenType {
    Lambda,
    Dot,
    Bracket(Direction),
    Identifier(String),
}

struct Lexer<O>
where
    O: Sink<Result<Token, LexError>>
{
    token_sink: O,
    state: State(Self::normal),
}

impl<O> Lexer<O> 
where
    O: Sink<Result<Token, LexError>>,
{
    fn new(token_sink: O) -> Lexer<O> {
        Lexer{
            token_sink,
        }
    }

    fn run<I>(&mut self, char_source: I) 
    where
        I: Source<char>,
    {
        while let Some(c) = char_source.take() {
            self.state = self.state(c);
            match c {
                '\n' => {
                    self.row += 1;
                    self.column = 0; 
                },
                _ => self.column += 1,
            }
    }
}

enum LexError {
    IdentifierError(String, Position),
    UnknownError,
}