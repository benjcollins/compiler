use std::fmt;

pub struct Program {
    blocks: Vec<Block>,
    variable_count: usize,
}

#[derive(Debug, Clone)]
pub struct Block {
    insts: Vec<Instruction>,
    exit: ExitInstruction,
    id: usize,
}

#[derive(Debug, Copy, Clone)]
pub struct BlockId {
    id: usize,
}

#[derive(Debug, Clone)]
pub enum Instruction {
    AddInt {
        dest: Var,
        a: Var,
        b: Var,
    },
    ConstantInt {
        dest: Var,
        constant: i32,
    },
    Call {
        function: BlockId,
    },
    Move {
        src: Var,
        dest: Var,
    }
}

#[derive(Debug, Clone)]
pub enum ExitInstruction {
    Branch {
        block: BlockId,
    },
    ConditionalBranch {
        cond: Var,
        block1: BlockId,
        block2: BlockId,
    },
    Return,
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Var {
    id: usize,
}

impl Program {
    pub fn new() -> Program {
        Program {
            blocks: vec![],
            variable_count: 0,
        }
    }
    pub fn new_variable(&mut self) -> Var {
        let variable = Var { id: self.variable_count };
        self.variable_count += 1;
        variable
    }
    pub fn get_variable_count(&self) -> usize {
        self.variable_count
    }
    fn submit_block(&mut self, block: Block) {
        let id = block.id;
        self.blocks[id] = block;
    }
    pub fn new_block(&mut self) -> Block {
        let block = Block { id: self.blocks.len(), insts: vec![], exit: ExitInstruction::Return };
        self.blocks.push(block.clone());
        block
    }
    pub fn get_block(&self, block: BlockId) -> &Block {
        &self.blocks[block.id]
    }
}

impl BlockId {
    pub fn entry() -> BlockId {
        BlockId { id: 0 }
    }
}

impl Block {
    pub fn get_id(&self) -> BlockId {
        BlockId { id: self.id }
    }
    pub fn add(&mut self, a: Var, b: Var, dest: Var) {
        self.insts.push(Instruction::AddInt { dest, a, b });
    }
    pub fn constant(&mut self, constant: i32, dest: Var) {
        self.insts.push(Instruction::ConstantInt { dest, constant });
    }
    pub fn call(&mut self, function: BlockId) {
        self.insts.push(Instruction::Call { function });
    }
    pub fn move_var(&mut self, src: Var, dest: Var) {
        self.insts.push(Instruction::Move { src, dest })
    }
    pub fn ret(mut self, program: &mut Program) {
        self.exit = ExitInstruction::Return;
        program.submit_block(self)
    }
    pub fn conditional_branch(mut self, cond: Var, block1: BlockId, block2: BlockId, program: &mut Program) {
        self.exit = ExitInstruction::ConditionalBranch { cond, block1, block2 };
        program.submit_block(self)
    }
    pub fn branch(mut self, block: BlockId, program: &mut Program) {
        self.exit = ExitInstruction::Branch { block };
        program.submit_block(self)
    }
    pub fn get_instructions(&self) -> &Vec<Instruction> {
        &self.insts
    }
    pub fn get_exit_instruction(&self) -> &ExitInstruction {
        &self.exit
    }
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (block_id, block) in self.blocks.iter().enumerate() {
            writeln!(f, "b{}:", block_id)?;
            for inst in block.insts.iter() {
                write!(f, "  ")?;
                match inst {
                    Instruction::AddInt { dest, a, b } => {
                        writeln!(f, "r{} = r{} + r{}", dest.id, a.id, b.id)?
                    }
                    Instruction::ConstantInt { dest, constant } => {
                        writeln!(f, "r{} = {}", dest.id, constant)?
                    }
                    Instruction::Call { function } => {
                        writeln!(f, "call b{}", function.id)?;
                    }
                    Instruction::Move { src, dest } => {
                        writeln!(f, "r{} = r{}", dest.id, src.id)?;
                    }
                }
            }
            write!(f, "  ")?;
            match block.exit {
                ExitInstruction::Return => {
                    write!(f, "return")?
                }
                ExitInstruction::ConditionalBranch { cond, block1, block2 } => {
                    write!(f, "if r{} goto b{} else goto b{}", cond.id, block1.id, block2.id)?;
                }
                ExitInstruction::Branch { block } => {
                    write!(f, "goto b{}", block.id)?;
                }
            }
            writeln!(f, "")?;
        }
        Ok(())
    }
}

impl Var {
    pub fn get_id(&self) -> usize {
        self.id
    }
}