use crate::ir::{Program, Function, Block, BlockId, Var, ExitInstruction, Instruction};
use std::rc::Rc;

#[derive(Debug)]
pub enum Graph {
    Empty,
    Node {
        var: Var,
        next: Rc<Graph>,
        interference: Vec<Rc<Graph>>,
    }
}

pub fn allocate_registers(program: &mut Program) {
    for function in program.get_functions() {
        println!("{:?}", build_interference_graph(function));
    }
}

fn count_uses_in_function(function: &mut Function) -> Vec<i32> {
    let mut uses = vec![0; function.get_variable_count()];
    let mut queue = vec![BlockId::main_block()];

    while let Some(block_id) = queue.pop() {
        count_uses_in_block(function.get_block(block_id), &mut uses, &mut queue)
    }

    for var in function.get_returns() {
        uses[var.get_id()] += 1;
    }
    
    uses
}

fn count_uses_in_block(block: &mut Block, uses: &mut Vec<i32>, queue: &mut Vec<BlockId>) {
    for inst in block.get_instructions() {
        match inst {
            Instruction::AddInt { a, b, .. } => {
                uses[a.get_id()] += 1;
                uses[b.get_id()] += 1;
            }
            Instruction::ConstantInt { .. } => {}
            Instruction::Phi { cond, a, b, .. } => {
                uses[cond.get_id()] += 1;
                uses[a.get_id()] += 1;
                uses[b.get_id()] += 1;
            }
            Instruction::Call { args, .. } => {
                for var in args {
                    uses[var.get_id()] += 1;
                }
            }
        }
    }
    match block.get_exit() {
        ExitInstruction::Branch { block } => {
            queue.push(*block);
        }
        ExitInstruction::ConditionalBranch { cond, block1, block2 } => {
            uses[cond.get_id()] += 1;
            queue.push(*block1);
            queue.push(*block2);
        }
        ExitInstruction::Return => {}
    }
}

fn build_interference_graph(function: &mut Function) -> Rc<Graph> {
    let mut uses = count_uses_in_function(function);
    let mut queue = vec![BlockId::main_block()];
    let mut graph = Rc::new(Graph::Empty);
    let mut live_vars = vec![];

    while let Some(block_id) = queue.pop() {
        graph = calculate_intference(function.get_block(block_id), graph, &mut live_vars, &mut uses, &mut queue);
    }
    graph
}

fn calculate_intference(block: &mut Block, mut graph: Rc<Graph>, live_vars: &mut Vec<Rc<Graph>>, uses: &mut Vec<i32>, queue: &mut Vec<BlockId>) -> Rc<Graph> {
    for inst in block.get_instructions() {
        match inst {
            Instruction::AddInt { dest, a, b } => {
                graph = init_var(*dest, live_vars, graph);
                use_var(*a, live_vars, uses);
                use_var(*b, live_vars, uses);
            }
            Instruction::ConstantInt { dest, .. } => {
                graph = init_var(*dest, live_vars, graph);
            }
            Instruction::Phi { cond, a, b, dest } => {
                graph = init_var(*dest, live_vars, graph);
                use_var(*a, live_vars, uses);
                use_var(*b, live_vars, uses);
                use_var(*cond, live_vars, uses);
            }
            Instruction::Call { args, returns, .. } => {
                for var in args {
                    use_var(*var, live_vars, uses)
                }
                for var in returns {
                    graph = init_var(*var, live_vars, graph)
                }
            }
        }
    }
    match block.get_exit() {
        ExitInstruction::Branch { block } => {
            queue.push(*block);
        }
        ExitInstruction::ConditionalBranch { cond, block1, block2 } => {
            use_var(*cond, live_vars, uses);
            queue.push(*block1);
            queue.push(*block2);
        }
        ExitInstruction::Return => {}
    }
    graph
}

fn init_var(var: Var, live_vars: &mut Vec<Rc<Graph>>, mut graph: Rc<Graph>) -> Rc<Graph> {
    graph = Rc::new(Graph::Node { var, next: graph, interference: live_vars.iter().cloned().collect() });
    live_vars.push(Rc::clone(&graph));
    graph
}

fn use_var(v: Var, live_vars: &mut Vec<Rc<Graph>>, uses: &mut Vec<i32>) {
    uses[v.get_id()] -= 1;
    if uses[v.get_id()] == 0 {
        let mut i = 0;
        while i < live_vars.len() {
            match &*live_vars[i] {
                Graph::Empty => i += 1,
                Graph::Node { var, .. } => {
                    if *var == v {
                        live_vars.swap_remove(i);
                    } else {
                        i += 1;
                    }
                }
            }
        }
    }
}