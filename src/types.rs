use std::{rc::Rc, cell::RefCell};
use crate::ir::{Program, Var, FunctionId, Block, Function};
use crate::ast::{Parsed, Expr};

#[derive(Debug, Clone)]
pub enum Type<'a, 'b> {
    Int(Var),
    Tuple(Vec<Type<'a, 'b>>),
    Func {
        pattern: &'b Parsed<'a, Expr<'a>>,
        expr: &'b Parsed<'a, Expr<'a>>,
        impls: Rc<RefCell<Vec<Implementation<'a, 'b>>>>,
    }
}

#[derive(Debug)]
pub struct Implementation<'a, 'b> {
    pub params: Type<'a, 'b>,
    pub returns: Type<'a, 'b>,
    pub function: FunctionId,
}

impl<'a, 'b> PartialEq for Type<'a, 'b> {
    fn eq(&self, other: &Type<'a, 'b>) -> bool {
        match (self, other) {
            (Type::Int(_), Type::Int(_)) => true,
            (Type::Tuple(a), Type::Tuple(b)) if a.len() == b.len() => true,
            (Type::Func { .. }, Type::Func { .. }) => true,
            _ => false,
        }
    }
}

impl<'a, 'b> Type<'a, 'b> {
    pub fn merge(a: Type<'a, 'b>, b: Type<'a, 'b>, program: &mut Program, block: &mut Block) -> Option<Type<'a, 'b>> {
        match (a, b) {
            (Type::Int(a), Type::Int(b)) => Some(Type::Int(block.phi(a, b, program))),
            (Type::Tuple(a), Type::Tuple(b)) if a.len() == b.len() => {
                let merged: Option<Vec<Type<'a, 'b>>> = a.iter().zip(b).map(|(a, b)| Type::merge(a.clone(), b.clone(), program, block)).collect();
                Some(Type::Tuple(merged?))
            }
            _ => None
        }
    }
    pub fn vars_as_vec(&self) -> Vec<Var> {
        match self {
            Type::Int(a) => vec![*a],
            Type::Tuple(types) => {
                let mut vec = vec![];
                for ty in types {
                    vec.extend(ty.vars_as_vec().iter().cloned())
                }
                vec
            }
            Type::Func { .. } => vec![],
        }
    }
    pub fn as_paramater(&self, program: &mut Program, function: &mut Function) -> Type<'a, 'b> {
        match self {
            Type::Int(_) => Type::Int(function.new_parameter(program)),
            Type::Tuple(types) => {
                let mut vec = vec![];
                for ty in types {
                    vec.push(ty.as_paramater(program, function));
                }
                Type::Tuple(vec)
            }
            Type::Func { pattern, expr, impls } => self.clone(),
        }
    }
}