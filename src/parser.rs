use common::{Direction, Sink, Source, State};
use lexer::{LexError, Position, Token, TokenType};

#[derive(Debug, PartialEq)]
enum Expression {
    Application(Vec<Expression>),
    Lambda(String, Box<Expression>),
    Variable(String),
}

enum Incomplete {
    Expressions(Vec<Expression>),
    Lambda(String, Vec<Expression>),
}

pub struct Parser
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
        loop {
            if self.parse_stack.len() == 1 {
                if let Incomplete::Expressions(v) = self.parse_stack.pop().unwrap() {
                    match Self::convert_to_expression(v) {
                        Some(e) => return e,
                        None => {
                            println!("expression was empty");
                            panic!();
                        },
                    }
                } else {
                    unreachable!();
                }
            } else if self.parse_stack.len() > 1 {
                match self.parse_stack.pop().unwrap() {
                    Incomplete::Lambda(i, mut v) => {
                        self.bubble_up_expression(Some(i), v);
                    }
                    Incomplete::Expressions(_) => {
                        println!("a closing bracket is missing");
                        panic!();
                    }
                }
            } else {
                unreachable!();
            }
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
                loop{
                    if self.parse_stack.len() > 1 {
                        match self.parse_stack.pop().unwrap() {
                            Incomplete::Expressions(mut v) => {
                                self.bubble_up_expression(None, v);
                                break;
                            },
                            Incomplete::Lambda(i, mut v) => {
                                self.bubble_up_expression(Some(i), v);
                            },
                        }
                    } else {
                        println!("Unexpected closing bracket at {:?}", t.position);
                        panic!();
                    }
                }
                State(Self::normal)
            }
            TokenType::Dot => {
                println!("A dot was found without a lambda at {:?}", t.position);
                panic!();
            }
            TokenType::Identifier(s) => {
                let expression = Expression::Variable(s);
                match self.parse_stack.pop() {
                    Some(mut incomplete) => {
                        match &mut incomplete {
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

    fn bubble_up_expression(&mut self, lambda_identifier: Option<String>, mut v: Vec<Expression>) {
        let inner_expression = match Self::convert_to_expression(v) {
            Some(e) => e,
            None => {
                println!("an expression in parenthesis was empty.");
                panic!();
            },
        };
        let expression = match lambda_identifier {
            Some(i) => Expression::Lambda(i, Box::new(inner_expression)),
            None => inner_expression,
        };
        let mut incomplete = self.parse_stack.pop().unwrap();
        match &mut incomplete {
            Incomplete::Expressions(e) => e.push(expression),
            Incomplete::Lambda(_, e) => e.push(expression),
        }
        self.parse_stack.push(incomplete);
    }

    fn convert_to_expression(mut v: Vec<Expression>) -> Option<Expression> {
        if v.len() > 1 {
            Some(Expression::Application(v))
        } else if let Some(e) = v.pop() {
            Some(e)
        } else {
            None
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

#[cfg(test)]
mod test {
    use std::collections::VecDeque;
    use super::{Parser, Expression};
    use lexer::Lexer;

    fn lex_parse_and_assert(string_slice: &str, expected: Expression) {
        let mut tokens = VecDeque::new();
        {
            let mut lexer = Lexer::new(&mut tokens);
            lexer.run(string_slice.to_string());
        }
        let mut parser = Parser::new();
        let expression = parser.run(tokens);
        assert_eq!(expression, expected);
    }

    #[test]
    fn nested_lambda() {
        let expected = Expression::Lambda("x".into(), 
            Box::new(
                Expression::Lambda("y".into(), 
                    Box::new(
                        Expression::Application(
                            vec![
                                Expression::Variable("x".into()),
                                Expression::Variable("y".into()),
                                Expression::Variable("z".into()),
                            ]
                        )
                    )
                )
            )
        );
        lex_parse_and_assert("λx y.x y z", expected);
    }

    #[test]
    fn single_lambda() {
        let expected = Expression::Lambda("xy".into(),
            Box::new(
                Expression::Variable("xyz".into())
            )
        );
        lex_parse_and_assert("λxy.xyz", expected);
    }

//     #[test]
//     fn lex_unicode_lambda() {
//         let expected = construct_expected!(
//             TokenType::Lambda, (0, 0), (0, 0);
//         );
//         lex_and_assert("λ", &expected);
//     }

//     #[test]
//     fn lex_non_unicode_lambda() {
//         let expected = construct_expected!(
//             TokenType::Lambda, (0, 0), (0, 0);
//         );
//         lex_and_assert("\\", &expected);
//     }

//     #[test]
//     fn lex_dot() {
//         let mut expected = VecDeque::new();
//         expected.push_back(Ok(Token::new(
//             TokenType::Dot,
//             Position::new(Point::new(0, 0), Point::new(0, 0)),
//         )));
//         lex_and_assert(".", &expected)
//     }

//     #[test]
//     fn lex_left_parenthesis() {
//         let mut expected = VecDeque::new();
//         expected.push_back(Ok(Token::new(
//             TokenType::Bracket(Direction::Left),
//             Position::new(Point::new(0, 0), Point::new(0, 0)),
//         )));
//         lex_and_assert("(", &expected)
//     }

//     #[test]
//     fn lex_right_parenthesis() {
//         let mut expected = VecDeque::new();
//         expected.push_back(Ok(Token::new(
//             TokenType::Bracket(Direction::Right),
//             Position::new(Point::new(0, 0), Point::new(0, 0)),
//         )));
//         lex_and_assert(")", &expected)
//     }

//     #[test]
//     fn lex_x_variable() {
//         let mut expected = VecDeque::new();
//         expected.push_back(Ok(Token::new(
//             TokenType::Identifier("x".into()),
//             Position::new(Point::new(0, 0), Point::new(0, 0)),
//         )));
//         lex_and_assert("x", &expected)
//     }

//     #[test]
//     fn lex_x0_variable() {
//         let mut expected = VecDeque::new();
//         expected.push_back(Ok(Token::new(
//             TokenType::Identifier("x0".into()),
//             Position::new(Point::new(0, 0), Point::new(0, 1)),
//         )));
//         lex_and_assert("x0", &expected)
//     }

//     #[test]
//     fn lex_x1_variable() {
//         let mut expected = VecDeque::new();
//         expected.push_back(Ok(Token::new(
//             TokenType::Identifier("x1".into()),
//             Position::new(Point::new(0, 0), Point::new(0, 1)),
//         )));
//         lex_and_assert("x1", &expected)
//     }

//     #[test]
//     fn lex_x2_variable() {
//         let mut expected = VecDeque::new();
//         expected.push_back(Ok(Token::new(
//             TokenType::Identifier("x2".into()),
//             Position::new(Point::new(0, 0), Point::new(0, 1)),
//         )));
//         lex_and_assert("x2", &expected)
//     }

//     #[test]
//     fn lex_lambda_x_dot_m_in_parenthesis() {
//         let expected = construct_expected!(
//             TokenType::Bracket(Direction::Left), (0, 0), (0, 0);
//             TokenType::Lambda, (0, 1), (0, 1);
//             TokenType::Identifier("x".into()), (0, 2), (0, 2);
//             TokenType::Dot, (0, 3), (0, 3);
//             TokenType::Identifier("M".into()), (0, 4), (0, 4);
//             TokenType::Bracket(Direction::Right), (0, 5), (0, 5);
//         );
//         lex_and_assert("(λx.M)", &expected);
//         lex_and_assert("(\\x.M)", &expected);
//     }

//     #[test]
//     fn lex_m_n_application_in_parenthesis() {
//         let expected = construct_expected!(
//             TokenType::Bracket(Direction::Left), (0, 0), (0, 0);
//             TokenType::Identifier("M".into()), (0, 1), (0, 1);
//             TokenType::Identifier("N".into()), (0, 3), (0, 3);
//             TokenType::Bracket(Direction::Right), (0, 4), (0, 4);
//         );
//         lex_and_assert("(M N)", &expected)
//     }

//     #[test]
//     fn lex_mn_variable_in_parenthesis() {
//         let expected = construct_expected!(
//             TokenType::Bracket(Direction::Left), (0, 0), (0, 0);
//             TokenType::Identifier("MN".into()), (0, 1), (0, 2);
//             TokenType::Bracket(Direction::Right), (0, 3), (0, 3);
//         );
//         lex_and_assert("(MN)", &expected)
//     }

//     #[test]
//     fn lex_lambda_x_dot_x() {
//         let expected = construct_expected!(
//             TokenType::Lambda, (0, 0), (0, 0);
//             TokenType::Identifier("x".into()), (0, 1), (0, 1);
//             TokenType::Dot, (0, 2), (0, 2);
//             TokenType::Identifier("x".into()), (0, 3), (0, 3);
//         );
//         lex_and_assert("λx.x", &expected);
//         lex_and_assert("\\x.x", &expected);
//     }

//     #[test]
//     fn lex_multiline_expression() {
//         use self::Direction::*;
//         use self::TokenType::*;
//         let expected = construct_expected!(
//             Lambda, (1, 0), (1, 0);
//             Identifier("x".into()), (1, 1), (1, 1);
//             Dot, (1, 2), (1, 2);
//             Bracket(Left), (1, 3), (1, 3);
//             Lambda, (2, 4), (2, 4);
//             Identifier("y".into()), (2, 5), (2, 5);
//             Dot, (2, 6), (2, 6);
//             Identifier("x".into()), (3, 8), (3, 8);
//             Identifier("y".into()), (4, 8), (4, 8);
//             Bracket(Right), (5, 0), (5, 0);
//             Identifier("z".into()), (5, 1), (5, 1);
//         );
//         lex_and_assert(
//             r#"
// λx.(
//     λy.
//         x
//         y
// )z
// "#,
//             &expected,
//         );
//     }
}
