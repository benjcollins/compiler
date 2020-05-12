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
use ir::{Function, Program, Block};

fn main() {
    let source = fs::read_to_string("example.txt").unwrap();
    let ast = parse_source(&source).unwrap();
    let mut program = Program::new();
    let mut function = Function::new();
    let mut block = Block::new();
    let _ = compile(&ast, &mut Scope::new(), &mut program, &mut function, &mut block).unwrap();
    let block_id = function.add_block(block);
    function.set_main(block_id);
    program.add_function(function);
    println!("{}", program);
}