use std::fmt;

pub struct Program {
    functions: Vec<Function>,
}

#[derive(Debug, Clone)]
pub struct Function {
    params: Vec<VarId>,
    returns: Vec<VarId>,
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
    id: usize,
}

#[derive(Debug, Copy, Clone)]
pub struct BlockId {
    id: usize,
}

#[derive(Debug, Clone)]
pub enum Instruction {
    AddInt {
        dest: VarId,
        a: VarId,
        b: VarId,
    },
    ConstantInt {
        dest: VarId,
        constant: i32,
    },
    Phi {
        cond: VarId,
        a: VarId,
        b: VarId,
        dest: VarId,
    },
    Call {
        function: FunctionId,
        args: Vec<VarId>,
        returns: Vec<VarId>,
    },
    Branch {
        block: BlockId,
    },
    BranchIf {
        cond: VarId,
        block: BlockId,
    },
    Return,
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct VarId {
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
}

impl Function {
    pub fn new() -> Function {
        Function { params: vec![], blocks: vec![], returns: vec![], variable_count: 0 }
    }
    fn new_local_variable(&mut self) -> VarId {
        let variable = VarId { id: self.variable_count };
        self.variable_count += 1;
        variable
    }
    pub fn new_parameter(&mut self) -> VarId {
        let var = self.new_local_variable();
        self.params.push(var);
        var
    }
    pub fn return_var(&mut self, var: VarId) {
        self.returns.push(var)
    }
    pub fn new_block(&mut self) -> Block {
        let block = Block { id: self.blocks.len(), insts: vec![] };
        self.blocks.push(Block { id: self.blocks.len(), insts: vec![] });
        block
    }
    pub fn submit_block(&mut self, block: Block) {
        let id = block.id;
        self.blocks[id] = block;
    }
}

impl Block {
    pub fn get_id(&self) -> BlockId {
        BlockId { id: self.id }
    }
    pub fn add_int(&mut self, a: VarId, b: VarId, function: &mut Function) -> VarId {
        let dest = function.new_local_variable();
        self.insts.push(Instruction::AddInt { dest, a, b });
        dest
    }
    pub fn constant_int(&mut self, constant: i32, function: &mut Function) -> VarId {
        let dest = function.new_local_variable();
        self.insts.push(Instruction::ConstantInt { dest, constant });
        dest
    }
    pub fn phi(&mut self, cond: VarId, a: VarId, b: VarId, function: &mut Function) -> VarId {
        let dest = function.new_local_variable();
        self.insts.push(Instruction::Phi { dest, cond, a, b });
        dest
    }
    pub fn call(&mut self, target_function_id: FunctionId, args: Vec<VarId>, return_count: usize, function: &mut Function) -> Vec<VarId> {
        let mut returns = Vec::new();
        for _ in 0..return_count {
            returns.push(function.new_local_variable())
        }
        self.insts.push(Instruction::Call { function: target_function_id, args, returns: returns.clone() });
        returns
    }
    pub fn ret(&mut self) {
        self.insts.push(Instruction::Return)
    }
    pub fn branch_if(&mut self, cond: VarId, block: BlockId) {
        self.insts.push(Instruction::BranchIf { cond, block })
    }
    pub fn branch(&mut self, block: BlockId) {
        self.insts.push(Instruction::Branch { block })
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
                        Instruction::Return => {
                            writeln!(f, "return")?
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
                        Instruction::BranchIf { cond, block } => {
                            writeln!(f, "if r{} goto b{}", cond.id, block.id)?;
                        }
                        Instruction::Branch { block } => {
                            writeln!(f, "goto b{}", block.id)?;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}