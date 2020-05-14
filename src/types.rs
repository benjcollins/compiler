use std::{rc::Rc, cell::RefCell, collections::{HashSet, HashMap}};
use crate::ir::{Var, FunctionId, Block, Function};
use crate::ast::{Parsed, Expr};

#[derive(Debug, Clone)]
pub enum Type<'a, 'b> {
    Int(Var),
    Bool(Var),
    Nothing,
    Either(Var, Vec<Type<'a, 'b>>),
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

impl<'a, 'b> Eq for Type<'a, 'b> {}

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
    pub fn get_used_vars(&self) -> Vec<Var> {
        let mut vars = HashSet::new();
        self.add_vars_to_map(&mut vars);
        vars.iter().cloned().collect::<Vec<Var>>()
    }
    pub fn add_vars_to_map(&self, map: &mut HashSet<Var>) {
        match self {
            Type::Int(var) => { map.insert(*var); },
            Type::Bool(var) => { map.insert(*var); },
            Type::Nothing => (),
            Type::Either(var, types) => {
                map.insert(*var);
                for ty in types {
                    ty.add_vars_to_map(map)
                }
            }
            Type::Tuple(types) => for ty in types {
                ty.add_vars_to_map(map)
            }
            Type::Func { .. } => (),
        }
    }
    pub fn map_to(&self, map: &HashMap<Var, Var>) -> Type<'a, 'b> {
        match self {
            Type::Int(var) => Type::Int(*map.get(var).unwrap()),
            Type::Bool(var) => Type::Bool(*map.get(var).unwrap()),
            Type::Nothing => Type::Nothing,
            Type::Either(var, types) => {
                Type::Either(*map.get(var).unwrap(), types.iter().map(|ty| ty.map_to(map)).collect())
            }
            Type::Tuple(types) => Type::Tuple(types.iter().map(|ty| ty.map_to(map)).collect()),
            Type::Func { .. } => self.clone(),
        }
    }
    pub fn as_parameter_ty(&self, function: &mut Function) -> Type<'a, 'b> {
        let vars = self.get_used_vars();
        let mut map = HashMap::new();
        for var in vars {
            map.insert(var, function.new_parameter());
        }
        self.map_to(&map)
    }
    pub fn return_ty(&self, function: &mut Function) {
        for var in self.get_used_vars() {
            function.return_var(var)
        }
    }
}