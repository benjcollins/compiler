use crate::{types::Type, ir::{Program, BlockId, Instruction, ExitInstruction, Var}};

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
    pub fn execute(&mut self, block: BlockId) {
        let mut block = self.program.get_block(block);
        for inst in block.get_instructions() {
            match inst {
                Instruction::AddInt { dest, a, b } => {
                    let a = self.get_register(*a);
                    let b = self.get_register(*b);
                    self.set_register(*dest, a + b)
                }
                Instruction::ConstantInt { dest, constant } => {
                    self.set_register(*dest, *constant);
                }
                Instruction::Call { function } => {
                    block = self.program.get_block(*function);
                    self.execute(*function);
                }
                Instruction::Move { src, dest } => {
                    let val = self.get_register(*src);
                    self.set_register(*dest, val);
                }
            }
        }
        match block.get_exit_instruction() {
            ExitInstruction::Branch { block: block_id } => { self.execute(*block_id) }
            ExitInstruction::ConditionalBranch { cond, block1, block2 } => {
                if self.get_register(*cond) != 0 {
                    self.execute(*block1);
                } else {
                    self.execute(*block2);
                }
            }
            ExitInstruction::Return => (),
        }
    }
    pub fn set_register(&mut self, reg: Var, value: i32) {
        self.register_file[reg.get_id()] = value;
    }
    pub fn get_register(&mut self, reg: Var) -> i32 {
        self.register_file[reg.get_id()]
    }
    pub fn format_ty<'b, 'c>(&mut self, ty: &Type<'b, 'c>) -> String {
        match ty {
            Type::Int(a) => format!("{}", self.get_register(*a)),
            Type::Bool(a) => format!("{}",  if self.get_register(*a) == 1 { "true" } else { "false" }),
            Type::Maybe(cond, ty) => if self.get_register(*cond) == 1 {
                format!("some({})", self.format_ty(ty))
            } else {
                "none".to_string()
            }
            Type::Tuple(types) => {
                let mut string = String::new();
                for ty in types {
                    string += self.format_ty(ty).as_str();
                }
                string
            }
            Type::Func { pattern, expr, impls } => unimplemented!(),
        }
    }
}