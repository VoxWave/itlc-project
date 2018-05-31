use parser::Expression;

struct NameGen {
    cur: usize,
}
impl NameGen {
    fn next(&mut self) -> String {
        let string = format!("#{}", self.cur);
        self.cur += 1;
        string
    }
}

fn rename(expr: Expression, from: String, to: String) -> Expression {
    match expr {
        Expression::Variable(i) => {
            if i == from {
                Expression::Variable(to)
            } else {
                Expression::Variable(i)
            }
        }
        Expression::Lambda(i, e) => {
            if i == from {
                Expression::Lambda(i, e)
            } else {
                Expression::Lambda(i, Box::new(rename(*e, from, to.clone())))
            }
        }
        Expression::Application(v) => Expression::Application(
            v.into_iter()
                .map(|e| rename(e, from.clone(), to.clone()))
                .collect(),
        ),
    }
}

fn alpha_conversion(expr: Expression, to: String) -> Expression {
    match expr {
        Expression::Lambda(i, e) => {
            if i == to {
                Expression::Lambda(i, e)
            } else {
                if identifier_exists(&*e, to.clone()) {
                    println!("tried to alpha convert into a free or binding variable");
                    panic!();
                }
                Expression::Lambda(to.clone(), Box::new(rename(*e, i, to)))
            }
        }
        _ => {
            println!("Tried to alpha convert something other than a lambda abstraction");
            panic!();
        }
    }
}

fn identifier_exists(expr: &Expression, ident: String) -> bool {
    let mut stack = Vec::new();
    stack.push(expr);
    while let Some(ref e) = stack.pop() {
        match *e {
            Expression::Application(v) => {
                for se in v {
                    stack.push(&se);
                }
            }
            Expression::Lambda(i, se) => {
                if i == &ident {
                    return true;
                }
                stack.push(&*se);
            }
            Expression::Variable(i) => if i == &ident {
                return true;
            },
        }
    }
    false
}

fn substitute(
    expr: Expression,
    from: String,
    to: Expression,
    name_gen: &mut NameGen,
) -> Expression {
    match expr {
        Expression::Variable(i) => {
            if i == from {
                to
            } else {
                Expression::Variable(i)
            }
        }
        Expression::Lambda(i, e) => {
            if i == from {
                Expression::Lambda(i, e)
            } else {
                let alpha_converted = alpha_conversion(Expression::Lambda(i, e), name_gen.next());
                match alpha_converted {
                    Expression::Lambda(i, e) => {
                        Expression::Lambda(i, Box::new(substitute(*e, from, to, name_gen)))
                    }
                    _ => unreachable!(),
                }
            }
        }
        Expression::Application(v) => Expression::Application(
            v.into_iter()
                .map(|e| substitute(e, from.clone(), to.clone(), name_gen))
                .collect(),
        ),
    }
}

fn beta_reduce(expr: Expression,) -> Expression {

}