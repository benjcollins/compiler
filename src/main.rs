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
use ir::{Function, Program};
use execute::VirtualMachine;

fn main() {
    let source = fs::read_to_string("example.txt").unwrap();
    let ast = parser::parse_source(&source).unwrap();
    println!("{}", ast.node);

    let mut program = Program::new();
    let mut function = Function::new();
    let mut block = function.new_block();
    let ty = compiler::compile(&ast, &mut Scope::new(), &mut program, &mut function, &mut block).unwrap();
    block.ret(&mut function);
    ty.return_ty(&mut function);
    let main_id = program.add_function(function);
    println!("{}", program);

    let function = program.get_function(main_id);

    let mut vm = VirtualMachine::new(&program);
    vm.execute(function);

    for var in function.get_returns() {
        println!("{}", vm.get_register(*var))
    }
}