use common::Direction;
use lexer::Lexer;
use parser::Parser;
use interpreter::interpret;

mod common;
mod file;
mod lexer;
mod parser;
mod interpreter;

fn main() {
    thread::spawn(move || {
        Lexer::new();
    });
}
