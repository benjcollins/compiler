use std::{rc::Rc, cell::RefCell};
use crate::ir::{Var, FunctionId, Block, Function};
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
    pub function: FunctionId,
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
    pub fn merge(cond: Var, a: &Type<'a, 'b>, b: &Type<'a, 'b>, function: &mut Function, block: &mut Block) -> Type<'a, 'b> {
        match (a, b) {
            (Type::Int(a), Type::Int(b)) => Type::Int(block.phi(cond, *a, *b, function)),
            (Type::Bool(a), Type::Bool(b)) => Type::Bool(block.phi(cond, *a, *b, function)),
            (Type::Tuple(atypes), Type::Tuple(btypes)) if atypes.len() == btypes.len() => {
                let mut types = vec![];
                for (a, b) in atypes.iter().zip(btypes) {
                    types.push(Type::merge(cond, a, b, function, block))
                }
                Type::Tuple(types)
            }
            _ => unimplemented!(),
        }
    }
    pub fn get_used_vars(&self) -> Vec<Var> {
        let mut vars = Vec::new();
        self.add_vars_to_vec(&mut vars);
        vars.iter().cloned().collect::<Vec<Var>>()
    }
    pub fn add_vars_to_vec(&self, map: &mut Vec<Var>) {
        match self {
            Type::Int(var) => { map.push(*var); },
            Type::Bool(var) => { map.push(*var); },
            Type::Maybe(var, ty) => {
                map.push(*var);
                ty.add_vars_to_vec(map);
            },
            Type::Tuple(types) => for ty in types {
                ty.add_vars_to_vec(map)
            }
            Type::Func { .. } => (),
        }
    }
    pub fn map_to(&self, mut vars: &[Var]) -> Type<'a, 'b> {
        match self {
            Type::Int(_) => Type::Int(vars[0]),
            Type::Bool(_) => Type::Bool(vars[0]),
            Type::Maybe(_, ty) => Type::Maybe(vars[0], Box::new(ty.map_to(&vars[1..]))),
            Type::Tuple(types) => {
                let mut vec = vec![];
                for ty in types {
                    vec.push(ty.map_to(&vars[..ty.size()]));
                    vars = &vars[ty.size()..];
                }
                Type::Tuple(vec)
            },
            Type::Func { .. } => self.clone(),
        }
    }
    pub fn size(&self) -> usize {
        match self {
            Type::Int(_) => 1,
            Type::Bool(_) => 1,
            Type::Maybe(_, ty) => 1 + ty.size(),
            Type::Tuple(types) => types.iter().map(|ty| ty.size()).sum(),
            Type::Func { .. } => 0,
        }
    }
    pub fn as_parameter_ty(&self, function: &mut Function) -> Type<'a, 'b> {
        let mut vars = vec![];
        for _ in 0..self.size() {
            vars.push(function.new_parameter());
        }
        self.map_to(&vars)
    }
    pub fn return_ty(&self, function: &mut Function) {
        for var in self.get_used_vars() {
            function.return_var(var)
        }
    }
}