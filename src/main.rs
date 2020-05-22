mod position;
mod parser;
mod ir;
mod ast;
mod compiler;
mod scope;
mod types;

use parser::parse_source;
use compiler::compile;
use scope::Scope;
use std::fs;
use ir::{Function, Program};

fn main() {
    let source = fs::read_to_string("example.txt").unwrap();
    let ast = parse_source(&source).unwrap();
    let mut program = Program::new();
    let mut function = Function::new();
    let mut block = function.new_block();
    let _ = compile(&ast, &mut Scope::new(), &mut program, &mut function, &mut block).unwrap();
    block.ret(&mut function);
    program.add_function(function);
    println!("{}", program);
}