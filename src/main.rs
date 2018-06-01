use common::Direction;
use lexer::Lexer;
use parser::Parser;
use interpreter::interpret;

mod common;
mod interpreter;
mod lexer;
mod parser;

fn main() {
    thread::spawn(move || {
        Lexer::new();
    });
}
