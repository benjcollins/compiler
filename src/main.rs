mod position;
mod parser;
mod ir;
mod ast;
mod compiler;
mod scope;
mod types;
mod execute;

use scope::Scope;
use std::fs;
use ir::Program;
use execute::VirtualMachine;

fn main() {
    let source = fs::read_to_string("example.txt").unwrap();
    let ast = parser::parse_source(&source).unwrap();
    println!("{}", ast.node);

    let mut program = Program::new();
    let mut block = program.new_block();
    let block_id = block.get_id();
    let ty = compiler::compile(&ast, &mut Scope::new(), &mut program, &mut block).unwrap();
    block.ret(&mut program);
    println!("{}", program);

    let mut vm = VirtualMachine::new(&program);
    vm.execute(block_id);
    println!("{}", vm.format_ty(&ty))
}