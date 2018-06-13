use common::{Direction, Sink, Source, State};
use lexer::{LexError, Position, Token, TokenType};

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Application(Vec<Expression>),
    Lambda(String, Box<Expression>),
    Variable(String),
}

enum Incomplete {
    Expressions(Vec<Expression>),
    Lambda(String, Vec<Expression>),
}

pub struct Parser {
    parse_stack: Vec<Incomplete>,
}

impl Parser {
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
                        }
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
                loop {
                    if self.parse_stack.len() > 1 {
                        match self.parse_stack.pop().unwrap() {
                            Incomplete::Expressions(mut v) => {
                                self.bubble_up_expression(None, v);
                                break;
                            }
                            Incomplete::Lambda(i, mut v) => {
                                self.bubble_up_expression(Some(i), v);
                            }
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
                    }
                    None => unreachable!(),
                }
                State(Self::normal)
            }
            TokenType::Lambda => State(Self::lambda),
        }
    }

    fn bubble_up_expression(&mut self, lambda_identifier: Option<String>, mut v: Vec<Expression>) {
        let inner_expression = match Self::convert_to_expression(v) {
            Some(e) => e,
            None => {
                println!("an expression in parenthesis was empty.");
                panic!();
            }
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
    use super::{Expression, Parser};
    use lexer::Lexer;
    use std::collections::VecDeque;

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

    fn lex_and_parse_only(string_slice: &str) {
        let mut tokens = VecDeque::new();
        {
            let mut lexer = Lexer::new(&mut tokens);
            lexer.run(string_slice.to_string());
        }
        let mut parser = Parser::new();
        let expression = parser.run(tokens);
    }

    #[test]
    fn nested_lambda() {
        let expected = Expression::Lambda(
            "x".into(),
            Box::new(Expression::Lambda(
                "y".into(),
                Box::new(Expression::Application(vec![
                    Expression::Variable("x".into()),
                    Expression::Variable("y".into()),
                    Expression::Variable("z".into()),
                ])),
            )),
        );
        lex_parse_and_assert("λx y.x y z", expected);
    }

    #[test]
    fn single_lambda_expression() {
        let expected =
            Expression::Lambda("xy".into(), Box::new(Expression::Variable("xyz".into())));
        lex_parse_and_assert("λxy.xyz", expected);
    }

    #[test]
    #[should_panic]
    fn try_to_parse_unicode_lambda() {
        lex_and_parse_only("λ");
    }

    #[test]
    #[should_panic]
    fn try_to_parse_non_unicode_lambda() {
        lex_and_parse_only("\\");
    }

    #[test]
    #[should_panic]
    fn try_to_parse_dot() {
        lex_and_parse_only(".");
    }

    #[test]
    #[should_panic]
    fn try_to_parse_left_parenthesis() {
        lex_and_parse_only("(");
    }

    #[test]
    #[should_panic]
    fn try_to_parse_right_parenthesis() {
        lex_and_parse_only(")");
    }

    #[test]
    fn parse_x_variable() {
        let expected = Expression::Variable("x".into());
        lex_parse_and_assert("x", expected);
    }

    #[test]
    fn parse_x0_variable() {
        let expected = Expression::Variable("x0".into());
        lex_parse_and_assert("x0", expected);
    }

    #[test]
    fn parse_x1_variable() {
        let expected = Expression::Variable("x1".into());
        lex_parse_and_assert("x1", expected);
    }

    #[test]
    fn parse_x2_in_parenthesis() {
        let expected = Expression::Variable("x2".into());
        lex_parse_and_assert("(((((x2)))))", expected);
    }

    #[test]
    fn parse_lambda_x_dot_m_in_parenthesis() {
        let expected = Expression::Lambda("x".into(), Box::new(Expression::Variable("M".into())));
        lex_parse_and_assert("(λx.M)", expected);
    }

    #[test]
    fn parse_m_n_application_in_parenthesis() {
        let expected = Expression::Application(vec![
            Expression::Variable("M".into()),
            Expression::Variable("N".into()),
        ]);
        lex_parse_and_assert("(M N)", expected);
    }

    #[test]
    fn parse_mn_variable_in_parenthesis() {
        let expected = Expression::Variable("MN".into());
        lex_parse_and_assert("(MN)", expected);
    }

    #[test]
    fn parse_lamda_x_dot_x() {
        let expected = Expression::Lambda("x".into(), Box::new(Expression::Variable("x".into())));
        lex_parse_and_assert("λx.x", expected);
    }

    #[test]
    #[should_panic]
    fn multiline_expression_with_the_wrong_expected_tree() {
        let expected = Expression::Application(vec![
            Expression::Lambda(
                "x".into(),
                Box::new(Expression::Lambda(
                    "y".into(),
                    Box::new(Expression::Application(vec![
                        Expression::Variable("x".into()),
                        Expression::Variable("y".into()),
                    ])),
                )),
            ),
            Expression::Variable("z".into()),
        ]);
        lex_parse_and_assert(
            r#"
λx.(
    λy.
        x
        y
)z
"#,
            expected,
        );
    }

    #[test]
    fn parse_multiline_expression() {
        let expected = Expression::Lambda(
            "x".into(),
            Box::new(Expression::Application(vec![
                Expression::Lambda(
                    "y".into(),
                    Box::new(Expression::Application(vec![
                        Expression::Variable("x".into()),
                        Expression::Variable("y".into()),
                    ])),
                ),
                Expression::Variable("z".into()),
            ])),
        );
        lex_parse_and_assert(
            r#"
λx.(
    λy.
        x
        y
)z
"#,
            expected,
        );
    }
}
