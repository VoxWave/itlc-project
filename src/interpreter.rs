use parser::Expression;

fn rename(expr: Expression, from: &str, to: String) -> Expression {
    match expr {
        Expression::Variable(i) => {
            if i == from {
                Expression::Variable(to)
            } else {
                Expression::Variable(i)
            }
        },
        Expression::Lambda(i, e) => {
            let identifier = if i == from {
                to.clone()
            } else {
                i
            };
            Expression::Lambda(identifier, Box::new(rename(*e, from, to.clone())))
        },
        Expression::Application(v) => {
            let mut app = Vec::new();
            for e in v {
                app.push(rename(e, from, to.clone()));
            }
            Expression::Application(app)
        },
    }
}