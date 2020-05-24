use std::fmt;

pub struct Program {
    functions: Vec<Function>,
}

#[derive(Debug, Clone)]
pub struct Function {
    params: Vec<Var>,
    returns: Vec<Var>,
    variable_count: usize,
    blocks: Vec<Block>,
}

#[derive(Debug, Copy, Clone)]
pub struct FunctionId {
    id: usize,
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
    Phi {
        cond: Var,
        a: Var,
        b: Var,
        dest: Var,
    },
    Call {
        function: FunctionId,
        args: Vec<Var>,
        returns: Vec<Var>,
    },
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
            functions: vec![],
        }
    }
    pub fn add_function(&mut self, function: Function) -> FunctionId {
        let id = self.functions.len();
        self.functions.push(function);
        FunctionId { id }
    }
    pub fn get_functions(&mut self) -> &mut [Function] {
        &mut self.functions
    }
}

impl Function {
    pub fn new() -> Function {
        Function { params: vec![], blocks: vec![], returns: vec![], variable_count: 0 }
    }
    fn new_local_variable(&mut self) -> Var {
        let variable = Var { id: self.variable_count };
        self.variable_count += 1;
        variable
    }
    pub fn new_parameter(&mut self) -> Var {
        let var = self.new_local_variable();
        self.params.push(var);
        var
    }
    pub fn return_var(&mut self, var: Var) {
        self.returns.push(var)
    }
    pub fn new_block(&mut self) -> Block {
        let block = Block { id: self.blocks.len(), insts: vec![], exit: ExitInstruction::Return };
        self.blocks.push(block.clone());
        block
    }
    fn submit_block(&mut self, block: Block) {
        let id = block.id;
        self.blocks[id] = block;
    }
    pub fn get_variable_count(&self) -> usize {
        self.variable_count
    }
    pub fn get_block(&mut self, id: BlockId) -> &mut Block {
        &mut self.blocks[id.id]
    }
    pub fn get_returns(&self) -> &[Var] {
        &self.returns
    }
}

impl BlockId {
    pub fn main_block() -> BlockId {
        BlockId { id: 0 }
    }
}

impl Block {
    pub fn get_id(&self) -> BlockId {
        BlockId { id: self.id }
    }
    pub fn add_int(&mut self, a: Var, b: Var, function: &mut Function) -> Var {
        let dest = function.new_local_variable();
        self.insts.push(Instruction::AddInt { dest, a, b });
        dest
    }
    pub fn constant_int(&mut self, constant: i32, function: &mut Function) -> Var {
        let dest = function.new_local_variable();
        self.insts.push(Instruction::ConstantInt { dest, constant });
        dest
    }
    pub fn phi(&mut self, cond: Var, a: Var, b: Var, function: &mut Function) -> Var {
        let dest = function.new_local_variable();
        self.insts.push(Instruction::Phi { dest, cond, a, b });
        dest
    }
    pub fn call(&mut self, target_function_id: FunctionId, args: Vec<Var>, return_count: usize, function: &mut Function) -> Vec<Var> {
        let mut returns = Vec::new();
        for _ in 0..return_count {
            returns.push(function.new_local_variable())
        }
        self.insts.push(Instruction::Call { function: target_function_id, args, returns: returns.clone() });
        returns
    }
    pub fn ret(mut self, function: &mut Function) {
        self.exit = ExitInstruction::Return;
        function.submit_block(self)
    }
    pub fn conditional_branch(mut self, cond: Var, block1: BlockId, block2: BlockId, function: &mut Function) {
        self.exit = ExitInstruction::ConditionalBranch { cond, block1, block2 };
        function.submit_block(self)
    }
    pub fn branch(mut self, block: BlockId, function: &mut Function) {
        self.exit = ExitInstruction::Branch { block };
        function.submit_block(self)
    }
    pub fn get_instructions(&mut self) -> &mut Vec<Instruction> {
        &mut self.insts
    }
    pub fn get_exit(&self) -> &ExitInstruction {
        &self.exit
    }
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (function_id, function) in self.functions.iter().enumerate() {
            write!(f, "f{} (", function_id)?;
            let mut iter = function.params.iter();
            if let Some(param) = iter.next() {
                write!(f, "r{}", param.id)?;
                for param in iter {
                    write!(f, ", r{}", param.id)?;
                }
            }
            write!(f, ")")?;
            let mut iter = function.returns.iter();
            if let Some(ret) = iter.next() {
                write!(f, " -> r{}", ret.id)?;
                for ret in iter {
                    write!(f, ", r{}", ret.id)?;
                }
            }
            writeln!(f, "")?;
            for (block_id, block) in function.blocks.iter().enumerate() {
                if block_id == 0 {
                    writeln!(f, "    main:")?;
                } else {
                    writeln!(f, "    b{}:", block_id)?;
                }
                for inst in block.insts.iter() {
                    write!(f, "        ")?;
                    match inst {
                        Instruction::AddInt { dest, a, b } => {
                            writeln!(f, "r{} = r{} + r{}", dest.id, a.id, b.id)?
                        }
                        Instruction::ConstantInt { dest, constant } => {
                            writeln!(f, "r{} = {}", dest.id, constant)?
                        }
                        Instruction::Phi { dest, cond, a, b } => {
                            writeln!(f, "r{} = phi(r{}, r{}, r{})", dest.id, cond.id, a.id, b.id)?
                        }
                        Instruction::Call { function, args, returns } => {
                            let mut iter = returns.iter();
                            if let Some(var) = iter.next() {
                                write!(f, "r{}", var.id)?;
                                for var in iter {
                                    write!(f, ", r{}", var.id)?;
                                }
                                write!(f, " = ")?;
                            }
                            write!(f, "call f{} (", function.id)?;
                            let mut iter = args.iter();
                            if let Some(var) = iter.next() {
                                write!(f, "r{}", var.id)?;
                                for var in iter {
                                    write!(f, ", r{}", var.id)?;
                                }
                            }
                            writeln!(f, ")")?;
                        }
                    }
                }
                write!(f, "        ")?;
                match block.exit {
                    ExitInstruction::Return => {
                        writeln!(f, "return")?
                    }
                    ExitInstruction::ConditionalBranch { cond, block1, block2 } => {
                        writeln!(f, "if r{} goto b{} else goto b{}", cond.id, block1.id, block2.id)?;
                    }
                    ExitInstruction::Branch { block } => {
                        writeln!(f, "goto b{}", block.id)?;
                    }
                }
                writeln!(f, "")?;
            }
        }
        Ok(())
    }
}

impl Var {
    pub fn get_id(&self) -> usize {
        self.id
    }
}