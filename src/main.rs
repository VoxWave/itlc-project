fn main() {
    println!("Hello, world!");
}

enum Token {
    Lambda,
    Dot,
    Bracket(Direction),
    Identifier(String),
}

enum Direction {
    Right, Left,
}

struct Lexer {

}