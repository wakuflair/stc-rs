mod parser;
mod ast;
mod utils;
mod transform;

use parser::*;
use ast::{AstNode};

fn main() {
    println!("Hello, world!");

    let lexer = Lexer::new("IF 2 + 3 THEN 4 - 3; END_IF");
    let r = parser::st::CompilationUnitsParser::new().parse(lexer).unwrap();

    let mut stringify = utils::StringifyVisitor::new(std::io::stdout());
    r.accept(&mut stringify);

    println!("{:?}", r);
}
