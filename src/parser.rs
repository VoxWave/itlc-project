
enum Expression {
    Expressions(Vec<Expression>),
    Lamda()
}

struct Parser<O>
where
    O: Sink<Result<Expression, ParserError>>,
{
    expression_sink: O,
    parse_stack: Vec<Vec<Expression>>,
}

impl Parser<O> 
where
    O: Sink<Result<Expression, ParserError>>,
{
    fn new() -> Self {

    }
}

enum ParserError {

}