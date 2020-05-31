use crate::ir::{Program, BlockId, Instruction, ExitInstruction, Var, Function};

pub struct VirtualMachine<'a> {
    register_file: Vec<i32>,
    program: &'a Program,
}

impl<'a> VirtualMachine<'a> {
    pub fn new(program: &'a Program) -> VirtualMachine<'a> {
        VirtualMachine {
            register_file: vec![0; program.get_variable_count()],
            program,
        }
    }
    pub fn execute(&mut self, function: &Function) {
        let mut block = function.get_block(BlockId::entry());
        loop {
            for inst in block.get_instructions() {
                match inst {
                    &Instruction::AddInt { dest, a, b } => {
                        let a = self.get_register(a);
                        let b = self.get_register(b);
                        self.set_register(dest, a + b)
                    }
                    &Instruction::ConstantInt { dest, constant } => {
                        self.set_register(dest, constant);
                    }
                    &Instruction::Phi { cond, a, b, dest } => {
                        let value = if self.get_register(cond) != 0 {
                            self.get_register(a)
                        } else {
                            self.get_register(b)
                        };
                        self.set_register(dest, value)
                    }
                    Instruction::Call { function, args, returns } => {
                        let function = self.program.get_function(*function);
                        for (param, arg) in function.get_params().iter().zip(args) {
                            let arg = self.get_register(*arg);
                            self.set_register(*param, arg);
                        }
                        self.execute(function);
                        for (ret, var) in function.get_returns().iter().zip(returns) {
                            let ret = self.get_register(*ret);
                            self.set_register(*var, ret);
                        }
                    }
                }
            }
            match block.get_exit_instruction() {
                &ExitInstruction::Branch { block: block_id } => { block = function.get_block(block_id) }
                &ExitInstruction::ConditionalBranch { cond, block1, block2 } => {
                    if self.get_register(cond) != 0 {
                        block = function.get_block(block1);
                    } else {
                        block = function.get_block(block2);
                    }
                }
                ExitInstruction::Return => { break }
            }
        }
    }
    pub fn set_register(&mut self, reg: Var, value: i32) {
        self.register_file[reg.get_id()] = value;
    }
    pub fn get_register(&mut self, reg: Var) -> i32 {
        self.register_file[reg.get_id()]
    }
}