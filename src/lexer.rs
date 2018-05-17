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

struct Lexer {
    token_sink: Sink<Result<Token, LexError>>
}

impl Lexer {
    fn new(token_sink: Sink<Result<Token, LexError>>) -> Lexer {
        Lexer{
            token_sink,
        }
    }

    fn run(char_source: Source<char>) {

    }
}

enum LexError {
    IdentifierError(String, Position),
    UnknownError,
}