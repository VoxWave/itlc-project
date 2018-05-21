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
        Point { row, column }
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
    O: Sink<Result<Token, LexError>>,
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
        Lexer {
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
                }
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
            }
            c if c.is_alphanumeric() => {
                self.starting_point = Some(Point {
                    row: self.row,
                    column: self.column,
                });
                self.buffer.push(c);
                State(Self::identifier)
            }
            w if w.is_whitespace() => State(Self::normal),

            _ => {
                let position = self.get_current_position();
                let error = LexError::InvalidCharacterError(c, position);
                self.token_sink.put(Err(error));
                State(Self::normal)
            }
        }
    }

    fn identifier(&mut self, c: char) -> State<Lexer<O>, char> {
        match c {
            c if c.is_alphanumeric() => {
                self.buffer.push(c);
                State(Self::identifier)
            }
            c if c.is_whitespace() || c == '\\' || c == 'λ' || c == '(' || c == ')'
                || c == '.' =>
            {
                let token_type = TokenType::Identifier(self.buffer.clone());
                self.buffer.clear();
                let mut position = self.get_current_position();
                position.ending_point.column -= 1;
                self.starting_point = None;
                let token = Token::new(token_type, position);
                self.token_sink.put(Ok(token));
                self.normal(c)
            }
            _ => {
                let position = self.get_current_position();
                let error = LexError::IdentifierError(
                    format!("Invalid character {} found in an identifier", c),
                    position,
                );
                self.token_sink.put(Err(error));
                State(Self::identifier)
            }
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
        Position {
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

#[cfg(test)]
mod test {
    use super::{LexError, Lexer, Point, Position, Token, TokenType};
    use common::Direction;
    use std::collections::VecDeque;

    macro_rules! construct_expected {
        ($($token:expr, $from:expr, $to:expr;)*) => {
            {
                let mut expected = VecDeque::new();
                $(
                    expected.push_back(Ok(Token::new(
                        $token,
                        Position::new(Point::new($from.0, $from.1), Point::new($to.0, $to.1)),
                    )));
                )*
                expected
            }
        };
    }

    fn lex_and_assert(string_slice: &str, expected: &VecDeque<Result<Token, LexError>>) {
        let mut sink = VecDeque::new();
        {
            let mut lexer = Lexer::new(&mut sink);
            lexer.run(string_slice.to_string());
        }
        assert_eq!(sink, *expected);
    }

    #[test]
    fn multi_character_identifier_test() {
        let expected = construct_expected!(
            TokenType::Lambda, (0, 0), (0, 0);
            TokenType::Identifier("xy".into()), (0, 1), (0, 2);
        );
        lex_and_assert("λxy.xyz", expected);
    }

    #[test]
    fn lex_unicode_lambda() {
        let expected = construct_expected!(
            TokenType::Lambda, (0, 0), (0, 0);
        );
        lex_and_assert("λ", &expected);
    }

    #[test]
    fn lex_non_unicode_lambda() {
        let expected = construct_expected!(
            TokenType::Lambda, (0, 0), (0, 0);
        );
        lex_and_assert("\\", &expected);
    }

    #[test]
    fn lex_dot() {
        let mut expected = VecDeque::new();
        expected.push_back(Ok(Token::new(
            TokenType::Dot,
            Position::new(Point::new(0, 0), Point::new(0, 0)),
        )));
        lex_and_assert(".", &expected)
    }

    #[test]
    fn lex_left_parenthesis() {
        let mut expected = VecDeque::new();
        expected.push_back(Ok(Token::new(
            TokenType::Bracket(Direction::Left),
            Position::new(Point::new(0, 0), Point::new(0, 0)),
        )));
        lex_and_assert("(", &expected)
    }

    #[test]
    fn lex_right_parenthesis() {
        let mut expected = VecDeque::new();
        expected.push_back(Ok(Token::new(
            TokenType::Bracket(Direction::Right),
            Position::new(Point::new(0, 0), Point::new(0, 0)),
        )));
        lex_and_assert(")", &expected)
    }

    #[test]
    fn lex_x_variable() {
        let mut expected = VecDeque::new();
        expected.push_back(Ok(Token::new(
            TokenType::Identifier("x".into()),
            Position::new(Point::new(0, 0), Point::new(0, 0)),
        )));
        lex_and_assert("x", &expected)
    }

    #[test]
    fn lex_x0_variable() {
        let mut expected = VecDeque::new();
        expected.push_back(Ok(Token::new(
            TokenType::Identifier("x0".into()),
            Position::new(Point::new(0, 0), Point::new(0, 1)),
        )));
        lex_and_assert("x0", &expected)
    }

    #[test]
    fn lex_x1_variable() {
        let mut expected = VecDeque::new();
        expected.push_back(Ok(Token::new(
            TokenType::Identifier("x1".into()),
            Position::new(Point::new(0, 0), Point::new(0, 1)),
        )));
        lex_and_assert("x1", &expected)
    }

    #[test]
    fn lex_x2_variable() {
        let mut expected = VecDeque::new();
        expected.push_back(Ok(Token::new(
            TokenType::Identifier("x2".into()),
            Position::new(Point::new(0, 0), Point::new(0, 1)),
        )));
        lex_and_assert("x2", &expected)
    }

    #[test]
    fn lex_lambda_x_dot_m_in_parenthesis() {
        let expected = construct_expected!(
            TokenType::Bracket(Direction::Left), (0, 0), (0, 0);
            TokenType::Lambda, (0, 1), (0, 1);
            TokenType::Identifier("x".into()), (0, 2), (0, 2);
            TokenType::Dot, (0, 3), (0, 3);
            TokenType::Identifier("M".into()), (0, 4), (0, 4);
            TokenType::Bracket(Direction::Right), (0, 5), (0, 5);
        );
        lex_and_assert("(λx.M)", &expected);
        lex_and_assert("(\\x.M)", &expected);
    }

    #[test]
    fn lex_m_n_application_in_parenthesis() {
        let expected = construct_expected!(
            TokenType::Bracket(Direction::Left), (0, 0), (0, 0);
            TokenType::Identifier("M".into()), (0, 1), (0, 1);
            TokenType::Identifier("N".into()), (0, 3), (0, 3);
            TokenType::Bracket(Direction::Right), (0, 4), (0, 4);
        );
        lex_and_assert("(M N)", &expected)
    }

    #[test]
    fn lex_mn_variable_in_parenthesis() {
        let expected = construct_expected!(
            TokenType::Bracket(Direction::Left), (0, 0), (0, 0);
            TokenType::Identifier("MN".into()), (0, 1), (0, 2);
            TokenType::Bracket(Direction::Right), (0, 3), (0, 3);
        );
        lex_and_assert("(MN)", &expected)
    }

    #[test]
    fn lex_lambda_x_dot_x() {
        let expected = construct_expected!(
            TokenType::Lambda, (0, 0), (0, 0);
            TokenType::Identifier("x".into()), (0, 1), (0, 1);
            TokenType::Dot, (0, 2), (0, 2);
            TokenType::Identifier("x".into()), (0, 3), (0, 3);
        );
        lex_and_assert("λx.x", &expected);
        lex_and_assert("\\x.x", &expected);
    }

    #[test]
    fn lex_multiline_expression() {
        use self::TokenType::*;
        use self::Direction::*;
        let expected = construct_expected!(
            Lambda, (1, 0), (1, 0);
            Identifier("x".into()), (1, 1), (1, 1);
            Dot, (1, 2), (1, 2);
            Bracket(Left), (1, 3), (1, 3);
            Lambda, (2, 4), (2, 4);
            Identifier("x".into()), (2, 5), (2, 5);
        );
        lex_and_assert(
r#"
λx.(
    λy.
        x
        y
)z
"#
, 
            expected
        );
    }

}
