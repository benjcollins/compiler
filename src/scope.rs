use std::{cell::RefCell, rc::Rc};
use crate::types::Type;

#[derive(Debug)]
pub struct Scope<'a, 'b> {
    node: Rc<RefCell<ScopeNode<'a, 'b>>>,
}

#[derive(Debug)]
pub enum ScopeNode<'a, 'b> {
    Empty,
    Definition {
        name: &'a str,
        ty: Type<'a, 'b>,
        previous: Rc<RefCell<ScopeNode<'a, 'b>>>,
    }
}

impl<'a, 'b> Scope<'a, 'b> {
    pub fn new() -> Scope<'a, 'b> {
        Scope { node: Rc::new(RefCell::new(ScopeNode::Empty)) }
    }
    pub fn define(&mut self, name: &'a str, ty: Type<'a, 'b>) {
        self.node = Rc::new(RefCell::new(ScopeNode::Definition { previous: Rc::clone(&self.node), name, ty }))
    }
    pub fn assign(&mut self, search: &'a str, new_ty: Type<'a, 'b>) {
        self.node.borrow_mut().assign(search, new_ty)
    }
    pub fn get(&self, search: &'a str) -> Option<Type<'a, 'b>> {
        self.node.borrow().get(search)
    }
}

impl<'a, 'b> ScopeNode<'a, 'b> {
    pub fn get(&self, search: &'a str) -> Option<Type<'a, 'b>> {
        match self {
            ScopeNode::Empty => None,
            ScopeNode::Definition { name, ty, previous } => {
                if name == &search {
                    Some(ty.clone())
                } else {
                    previous.borrow().get(search)
                }
            }
        }
    }
    pub fn assign(&mut self, search: &'a str, new_ty: Type<'a, 'b>) {
        match self {
            ScopeNode::Empty => (),
            ScopeNode::Definition { name, ty, previous } => {
                if name == &search {
                    *ty = new_ty;
                } else {
                    previous.borrow_mut().assign(search, new_ty)
                }
            }
        }
    }
}