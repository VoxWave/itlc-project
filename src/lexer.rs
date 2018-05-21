use common::{Direction, Sink, Source, State};
use std::fmt;

#[derive(PartialEq)]
pub struct Token {
    token_type: TokenType,
    position: Position,
}

impl Token {
    fn new(token_type: TokenType, position: Position) -> Token {
        Token {
            token_type,
            position,
        }
    }
}

impl fmt::Debug for Token {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(
            fmt,
            "{:?} at from: {}:{} to: {}:{}",
            self.token_type,
            self.position.starting_point.row,
            self.position.starting_point.column,
            self.position.ending_point.row,
            self.position.ending_point.column,
        )
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Point {
    pub row: usize,
    pub column: usize,
}

impl Point {
    fn new(row: usize, column: usize) -> Self {
        Point {
            row,
            column,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Position {
    starting_point: Point,
    ending_point: Point,
}

impl Position {
    fn new(starting_point: Point, ending_point: Point) -> Self {
        Position {
            starting_point,
            ending_point,
        }
    }
}

#[derive(Debug, PartialEq)]
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
    buffer: String,
    starting_point: Option<Point>,
    row: usize,
    column: usize,
}

impl<O> Lexer<O> 
where
    O: Sink<Result<Token, LexError>>,
{
    fn new(token_sink: O) -> Lexer<O> {
        Lexer{
            token_sink,
            buffer: String::new(),
            starting_point: None,
            row: 0,
            column: 0,
        }
    }

    fn run<I>(&mut self, mut char_source: I) 
    where
        I: Source<char>,
    {   
        let mut state = State(Self::normal);
        while let Some(c) = char_source.take() {
            state = state(self, c);
            match c {
                '\n' => {
                    self.row += 1;
                    self.column = 0; 
                },
                _ => self.column += 1,
            }
        }
        if !self.buffer.is_empty() {
            let token_type = TokenType::Identifier(self.buffer.clone());
            self.buffer.clear();
            let mut position = self.get_current_position();
            position.ending_point.column -= 1;
            self.starting_point = None;
            let token = Token::new(token_type, position);
            self.token_sink.put(Ok(token));
        }
    }

    fn normal(&mut self, c: char) -> State<Lexer<O>, char> {
        use self::TokenType::*;
        use Direction::*;
        match c {
            '\\' | 'λ' | '(' | ')' | '.' => {
                let token_type = match c {
                    '\\' | 'λ' => Lambda,
                    '(' => Bracket(Left),
                    ')' => Bracket(Right),
                    '.' => Dot,
                    _ => unreachable!(),
                };
                let position = self.get_current_position();
                let token = Token::new(token_type, position);
                self.token_sink.put(Ok(token));
                State(Self::normal)
            },
            c if c.is_alphanumeric() => {
                self.starting_point = Some(Point {
                    row: self.row,
                    column: self.column,
                });
                self.buffer.push(c);
                State(Self::identifier)
            },
            w if w.is_whitespace() => State(Self::normal),

            _ => {
                let position = self.get_current_position();
                let error = LexError::InvalidCharacterError(c, position);
                self.token_sink.put(Err(error));
                State(Self::normal)
            },
        }
    }

    fn identifier(&mut self, c: char) -> State<Lexer<O>, char> {
        match c {
            c if c.is_alphanumeric() => {
                self.buffer.push(c);
                State(Self::identifier)
            },
            c if c.is_whitespace() || c == '\\' || c == 'λ' || c == '(' || c == ')' || c == '.' => {
                let token_type = TokenType::Identifier(self.buffer.clone());
                self.buffer.clear();
                let mut position = self.get_current_position();
                position.ending_point.column -= 1;
                self.starting_point = None;
                let token = Token::new(token_type, position);
                self.token_sink.put(Ok(token));
                self.normal(c)
            },
            _ => {
                let position = self.get_current_position();
                let error = LexError::IdentifierError(format!("Invalid character {} found in an identifier", c), position);
                self.token_sink.put(Err(error));
                State(Self::identifier)
            },
        }
    }

    fn get_current_position(&mut self) -> Position {
        let starting_point = match self.starting_point {
            Some(point) => point,
            None => Point {
                row: self.row,
                column: self.column,
            },
        };
        let ending_point = Point {
            row: self.row,
            column: self.column,
        };
        Position{
            starting_point,
            ending_point,
        }
    }
}

#[derive(Debug, PartialEq)]
enum LexError {
    IdentifierError(String, Position),
    InvalidCharacterError(char, Position),
}

#[test]
fn basic_expression_lex_test() {
    use std::collections::VecDeque;
    let mut sink = VecDeque::new();
    {
        let mut lexer = Lexer::new(&mut sink);
        let string = String::from("λx.x");
        lexer.run(string);
    }
    let mut expected = VecDeque::new();
    expected.push_back(Ok(Token::new(TokenType::Lambda, Position::new(Point::new(0, 0), Point::new(0, 0)))));
    expected.push_back(Ok(Token::new(TokenType::Identifier("x".into()), Position::new(Point::new(0, 1), Point::new(0, 1)))));
    expected.push_back(Ok(Token::new(TokenType::Dot, Position::new(Point::new(0, 2), Point::new(0, 2)))));
    expected.push_back(Ok(Token::new(TokenType::Identifier("x".into()), Position::new(Point::new(0, 3), Point::new(0, 3)))));
    assert_eq!(sink, expected);
}

#[test]
fn basic_expression_with_non_unicode_lambda_lex_test() {
    use std::collections::VecDeque;
    let mut sink = VecDeque::new();
    {
        let mut lexer = Lexer::new(&mut sink);
        let raw = r#"\x.x"#;
        let string = String::from(raw);
        lexer.run(string);
    }
    let mut expected = VecDeque::new();
    expected.push_back(Ok(Token::new(TokenType::Lambda, Position::new(Point::new(0, 0), Point::new(0, 0)))));
    expected.push_back(Ok(Token::new(TokenType::Identifier("x".into()), Position::new(Point::new(0, 1), Point::new(0, 1)))));
    expected.push_back(Ok(Token::new(TokenType::Dot, Position::new(Point::new(0, 2), Point::new(0, 2)))));
    expected.push_back(Ok(Token::new(TokenType::Identifier("x".into()), Position::new(Point::new(0, 3), Point::new(0, 3)))));
    assert_eq!(sink, expected);
}

#[test]
fn multi_character_identifier_test() {
    use std::collections::VecDeque;
    let mut sink = VecDeque::new();
    {
        let mut lexer = Lexer::new(&mut sink);
        let string = String::from("λxy.xyz");
    }
}