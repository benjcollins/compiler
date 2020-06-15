use std::{rc::Rc, cell::RefCell};
use crate::ir::{Var, Block, Program, BlockId};
use crate::ast::{Parsed, Expr};

#[derive(Debug, Clone)]
pub enum Type<'a, 'b> {
    Int(Var),
    Bool(Var),
    Maybe(Var, Box<Type<'a, 'b>>),
    Tuple(Vec<Type<'a, 'b>>),
    Func {
        pattern: &'b Parsed<'a, Expr<'a>>,
        expr: &'b Parsed<'a, Expr<'a>>,
        impls: Rc<RefCell<Vec<Implementation<'a, 'b>>>>,
    }
}

#[derive(Debug)]
pub struct Implementation<'a, 'b> {
    pub param_ty: Type<'a, 'b>,
    pub return_ty: Type<'a, 'b>,
    pub entry_block: BlockId,
}

impl<'a, 'b> PartialEq for Type<'a, 'b> {
    fn eq(&self, other: &Type<'a, 'b>) -> bool {
        match (self, other) {
            (Type::Int(_), Type::Int(_)) => true,
            (Type::Bool(_), Type::Bool(_)) => true,
            (Type::Tuple(atypes), Type::Tuple(btypes)) if atypes.len() == btypes.len() => {
                for (a, b) in atypes.iter().zip(btypes) {
                    if a != b {
                        return false
                    }
                }
                return true
            }
            _ => false,
        }
    }
}

impl<'a, 'b> Eq for Type<'a, 'b> {}

impl<'a, 'b> Type<'a, 'b> {
    pub fn merge(&self, other: &Type<'a, 'b>, block: &mut Block) {
        match (self, other) {
            (Type::Int(a), Type::Int(b)) => {
                block.move_var(*b, *a);
            }
            (Type::Bool(a), Type::Bool(b)) => {
                block.move_var(*b, *a);
            }
            (Type::Tuple(atypes), Type::Tuple(btypes)) if atypes.len() == btypes.len() => {
                let mut types = vec![];
                for (a, b) in atypes.iter().zip(btypes) {
                    types.push(Type::merge(a, b, block))
                }
            }
            _ => unimplemented!(),
        }
    }
    pub fn duplicate(&self, program: &mut Program) -> Type<'a, 'b> {
        match self {
            Type::Int(_) => Type::Int(program.new_variable()),
            Type::Bool(_) => Type::Bool(program.new_variable()),
            Type::Maybe(_, ty) => Type::Maybe(program.new_variable(), ty.clone()),
            Type::Tuple(types) => Type::Tuple(types.iter().map(|ty| ty.duplicate(program)).collect()),
            _ => unimplemented!(),
        }
    }
    pub fn copy(&self, program: &mut Program, block: &mut Block) -> Type<'a, 'b> {
        let clone = self.duplicate(program);
        clone.merge(self, block);
        clone
    }
}