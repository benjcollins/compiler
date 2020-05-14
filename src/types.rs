use std::{rc::Rc, cell::RefCell, slice::Iter};
use crate::ir::{Var, FunctionId, Block, Function};
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
    pub param_ty: Type<'a, 'b>,
    pub return_ty: Type<'a, 'b>,
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
    pub fn merge(a: Type<'a, 'b>, b: Type<'a, 'b>, function: &mut Function, block: &mut Block) -> Option<Type<'a, 'b>> {
        match (a, b) {
            (Type::Int(a), Type::Int(b)) => Some(Type::Int(block.phi(a, b, function))),
            (Type::Tuple(a), Type::Tuple(b)) if a.len() == b.len() => {
                let merged: Option<Vec<Type<'a, 'b>>> = a.iter().zip(b).map(|(a, b)| Type::merge(a.clone(), b.clone(), function, block)).collect();
                Some(Type::Tuple(merged?))
            }
            _ => None
        }
    }
    pub fn return_type(&self, function: &mut Function) {
        match self {
            Type::Int(a) => function.return_var(*a),
            Type::Tuple(types) => for ty in types {
                ty.return_type(function)
            }
            Type::Func { .. } => (),
        }
    }
    pub fn cast(&self, vars: &mut Iter<Var>) -> Type<'a, 'b> {
        match self {
            Type::Int(_) => Type::Int(*vars.next().unwrap()),
            Type::Tuple(types) => {
                let mut vec = vec![];
                for ty in types {
                    vec.push(ty.cast(vars))
                }
                Type::Tuple(vec)
            }
            Type::Func { .. } => self.clone(),
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
    pub fn as_paramater(&self, function: &mut Function) -> Type<'a, 'b> {
        match self {
            Type::Int(_) => Type::Int(function.new_parameter()),
            Type::Tuple(types) => {
                let mut vec = vec![];
                for ty in types {
                    vec.push(ty.as_paramater(function));
                }
                Type::Tuple(vec)
            }
            Type::Func { pattern, expr, impls } => self.clone(),
        }
    }
}