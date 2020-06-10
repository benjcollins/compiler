use crate::position::Position;
use std::fmt;

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
    pub fn write(&self, f: &mut fmt::Formatter, align: i32) -> fmt::Result {
        match self {
            Expr::IntLiteral(src) => write!(f, "{}", src)?,
            Expr::BoolLiteral(src) => write!(f, "{}", src)?,
            Expr::Ident(src) => write!(f, "{}", src)?,
            Expr::Tuple { exprs } => {
                let mut iter = exprs.iter();
                write!(f, "(")?;
                if let Some(expr) = iter.next() {
                    expr.node.write(f, align)?;
                    for expr in iter {
                        write!(f, ", ")?;
                        expr.node.write(f, align)?;
                    }
                }
                write!(f, ")")?;
            }
            Expr::Block { exprs, last } => {
                writeln!(f, "{{")?;
                for expr in exprs {
                    write!(f, "{}", Indent(align+4))?;
                    expr.node.write(f, align+4)?;
                    writeln!(f, "")?;
                }
                write!(f, "{}", Indent(align+4))?;
                last.node.write(f, align+4)?;
                writeln!(f, "\n}}")?;
            }
            Expr::Func { name, pattern, expr } => {
                write!(f, "fn")?;
                match name {
                    Some(name) => write!(f, " {}", name)?,
                    None => {},
                }
                write!(f, "(")?;
                pattern.node.write(f, align)?;
                write!(f, ") ")?;
                expr.node.write(f, align)?;
                writeln!(f, "")?;
            }
            Expr::Binary { left, right, op } => {
                left.node.write(f, align)?;
                write!(f, "{}", match op {
                    BinaryOp::Plus => " + ",
                    BinaryOp::Bracket => " (",
                    BinaryOp::SingleEquals => " = ",
                    BinaryOp::Else => " else ",
                })?;
                right.node.write(f, align)?;
                write!(f, "{}", if let BinaryOp::Bracket = op { ")" } else { "" })?;
            }
            Expr::If { cond, conc } => {
                write!(f, "if ")?;
                cond.node.write(f, align)?;
                write!(f, " ")?;
                conc.node.write(f, align)?;
            }
        };
        Ok(())
    }
}

struct Indent(i32);

impl fmt::Display for Indent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for _ in 0..self.0 {
            write!(f, " ")?;
        }
        Ok(())
    }
}

impl<'a> fmt::Display for Expr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.write(f, 0)
    }
}