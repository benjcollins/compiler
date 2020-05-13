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
    main: BlockId,
}

#[derive(Debug, Copy, Clone)]
pub struct FunctionId {
    id: usize,
}

#[derive(Debug, Clone)]
pub struct Block {
    insts: Vec<Instruction>,
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
        a: Var,
        b: Var,
        dest: Var,
    },
    Call {
        function: FunctionId,
        args: Vec<Var>,
        returns: Vec<Var>,
    },
    Return,
}

#[derive(Debug, Copy, Clone)]
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
}

impl Function {
    pub fn new() -> Function {
        Function { params: vec![], blocks: vec![], returns: vec![], variable_count: 0, main: BlockId { id: 0 } }
    }
    pub fn set_main(&mut self, block: BlockId) {
        self.main = block
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
    pub fn add_block(&mut self, block: Block) -> BlockId {
        let id = self.blocks.len();
        self.blocks.push(block);
        BlockId { id }
    }
}

impl Block {
    pub fn new() -> Block {
        Block { insts: vec![] }
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
    pub fn phi(&mut self, a: Var, b: Var, function: &mut Function) -> Var {
        let dest = function.new_local_variable();
        self.insts.push(Instruction::Phi { dest, a, b });
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
    pub fn ret(&mut self) {
        self.insts.push(Instruction::Return)
    }
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (function_id, function) in self.functions.iter().enumerate() {
            write!(f, "fn {} (", function_id)?;
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
                if block_id == function.main.id {
                    writeln!(f, "    main:")?;
                } else {
                    writeln!(f, "    {}:", block_id)?;
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
                        Instruction::Phi { dest, a, b } => {
                            writeln!(f, "r{} = phi(r{}, r{})", dest.id, a.id, b.id)?
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
                            write!(f, "call {} (", function.id)?;
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
            }
        }
        Ok(())
    }
}