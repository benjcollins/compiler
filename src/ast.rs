use crate::position::Position;

#[derive(Debug)]
pub struct Parsed<'a, T> {
    start: Position<'a>,
    end: Position<'a>,
    pub node: T,
}

#[derive(Debug)]
pub enum Expr<'a> {
    IntLiteral(&'a str),
    BoolLiteral(&'a str),
    Ident(&'a str),
    Tuple {
        exprs: Vec<Parsed<'a, Expr<'a>>>,
    },
    Block {
        exprs: Vec<Parsed<'a, Expr<'a>>>,
        last: Box<Parsed<'a, Expr<'a>>>,
    },
    Func {
        name: Option<&'a str>,
        pattern: Box<Parsed<'a, Expr<'a>>>,
        expr: Box<Parsed<'a, Expr<'a>>>,
    },
    Binary {
        left: Box<Parsed<'a, Expr<'a>>>,
        right: Box<Parsed<'a, Expr<'a>>>,
        op: BinaryOp,
    },
    If {
        cond: Box<Parsed<'a, Expr<'a>>>,
        conc: Box<Parsed<'a, Expr<'a>>>,
    }
}

#[derive(Debug)]
pub enum BinaryOp {
    Plus,
    Bracket,
    SingleEquals,
    Else,
}

impl<'a, T> Parsed<'a, T> {
    pub fn new(start: Position<'a>, end: Position<'a>, node: T) -> Parsed<'a, T> {
        Parsed { start, end, node }
    }
    pub fn start(&self) -> Position<'a> {
        self.start
    }
    pub fn end(&self) -> Position<'a> {
        self.end
    }
    pub fn get_source(&self) -> &'a str {
        Position::slice(self.start(), self.end())
    }
    pub fn get_node(&self) -> &T {
        &self.node
    }
}

impl<'a> Expr<'a> {
    pub fn new_binary(left: Parsed<'a, Expr<'a>>, right: Parsed<'a, Expr<'a>>, op: BinaryOp) -> Parsed<'a, Expr<'a>> {
        Parsed::new(left.start(), right.end(), Expr::Binary { left: Box::new(left), right: Box::new(right), op })
    }
    pub fn new_tuple(left: Parsed<'a, Expr<'a>>, right: Parsed<'a, Expr<'a>>) -> Parsed<'a, Expr<'a>> {
        Parsed::new(left.start(), right.end(), match left.node {
            Expr::Tuple { mut exprs } => {
                exprs.push(right);
                Expr::Tuple { exprs }
            }
            _ => Expr::Tuple { exprs: vec![left, right] }
        })
    }
    pub fn new_block(left: Parsed<'a, Expr<'a>>, right: Parsed<'a, Expr<'a>>) -> Parsed<'a, Expr<'a>> {
        Parsed::new(left.start(), right.end(), match left.node {
            Expr::Block { mut exprs, last } => {
                exprs.push(*last);
                Expr::Block { exprs, last: Box::new(right) }
            }
            _ => Expr::Block { exprs: vec![left], last: Box::new(right) }
        })
    }
}