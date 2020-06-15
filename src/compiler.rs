use crate::ast::{Parsed, Expr, BinaryOp};
use crate::{scope::Scope, ir::{Program, Block}, types::{Implementation, Type}};
use std::{cell::RefCell, rc::Rc};

#[derive(Debug)]
pub struct CompileError<'a> {
    source: &'a str,
    ty: CompileErrorType,
}

#[derive(Debug)]
pub enum CompileErrorType {
    TypeError,
    UndefinedVariable,
}

impl<'a> CompileError<'a> {
    pub fn type_error(source: &'a str) -> CompileError<'a> {
        CompileError { source, ty: CompileErrorType::TypeError }
    }
    pub fn undefined_variable(source: &'a str) -> CompileError<'a> {
        CompileError { source, ty: CompileErrorType::UndefinedVariable }
    }
}

pub fn compile<'a, 'b>(expr: &'b Parsed<'a, Expr<'a>>, scope: &mut Scope<'a, 'b>, program: &mut Program, block: &mut Block) -> Result<Type<'a, 'b>, CompileError<'a>> {
    match expr.get_node() {
        Expr::IntLiteral(source) => {
            let value = source.parse::<i32>().unwrap();
            let dest = program.new_variable();
            block.constant(value, dest);
            Ok(Type::Int(dest))
        }
        Expr::BoolLiteral(source) => {
            let value = if *source == "true" { 1 } else { 0 };
            let dest = program.new_variable();
            block.constant(value, dest);
            Ok(Type::Bool(dest))
        }
        Expr::Binary { left, right, op } => match op {
            BinaryOp::Plus => {
                let left = compile(left, scope, program, block)?;
                let right = compile(right, scope, program, block)?;
                match (left, right) {
                    (Type::Int(a), Type::Int(b)) => {
                        let dest = program.new_variable();
                        block.add(a, b, dest);
                        Ok(Type::Int(dest))
                    },
                    _ => Err(CompileError::type_error(expr.get_source()))
                }
            }
            BinaryOp::Bracket => {
                match compile(left, scope, program, block)? {
                    Type::Func { pattern, expr, impls } => {
                        let argument_ty = compile(right, scope, program, block)?;
                        for imp in impls.borrow().iter() {
                            if imp.param_ty == argument_ty {
                                imp.param_ty.merge(&argument_ty, block);
                                block.call(imp.entry_block);
                                return Ok(imp.return_ty.copy(program, block));
                            }
                        }
                        let mut new_block = program.new_block();
                        let mut function_scope = Scope::new();
                        match_pattern(pattern, argument_ty.clone(), &mut function_scope)?;
                        let return_ty = compile(expr, &mut function_scope, program, &mut new_block)?;
                        let block_id = new_block.get_id();
                        new_block.ret(program);
                        block.call(block_id);
                        let return_ty = return_ty.copy(program, block);
                        let imp = Implementation { param_ty: argument_ty, return_ty: return_ty.clone(), entry_block: block_id };
                        impls.borrow_mut().push(imp);
                        Ok(return_ty)
                    }
                    _ => Err(CompileError::type_error(expr.get_source()))
                }
            },
            BinaryOp::SingleEquals => {
                let ty = compile(right, scope, program, block)?;
                match_pattern(left, ty.clone(), scope)?;
                Ok(ty)
            }
            BinaryOp::Else => {
                if let Type::Maybe(tag, ty) = compile(left, scope, program, block)? {
                    let mut cond_block = program.new_block();
                    let exit_block = program.new_block();
                    block.clone().conditional_branch(tag, exit_block.get_id(), cond_block.get_id(), program);
                    let conc = compile(right, scope, program, &mut cond_block)?;
                    cond_block.branch(exit_block.get_id(), program);
                    *block = exit_block;
                    ty.merge(&conc, block);
                    Ok(*ty)
                } else {
                    Err(CompileError::type_error(expr.get_source()))
                }
            },
        }
        Expr::If { cond, conc } => {
            if let Type::Bool(cond) = compile(cond, scope, program, block)? {
                let mut cond_block = program.new_block();
                let exit_block = program.new_block();
                block.clone().conditional_branch(cond, cond_block.get_id(), exit_block.get_id(), program);
                let conc = compile(conc, scope, program, &mut cond_block)?;
                cond_block.branch(exit_block.get_id(), program);
                *block = exit_block;
                Ok(Type::Maybe(cond, Box::new(conc)))
            } else {
                Err(CompileError::type_error(expr.get_source()))
            }
        },
        Expr::Tuple { exprs } => {
            let mut types = Vec::new();
            for expr in exprs {
                types.push(compile(expr, scope, program, block)?)
            }
            Ok(Type::Tuple(types))
        }
        Expr::Block { exprs, last } => {
            for expr in exprs {
                compile(expr, scope, program, block)?;
            }
            compile(last, scope, program, block)
        },
        Expr::Ident(source) => match scope.get(source) {
            Some(ty) => Ok(ty),
            None => Err(CompileError::undefined_variable(expr.get_source())),
        }
        Expr::Func { name, pattern, expr } => {
            let func = Type::Func { pattern, expr, impls: Rc::new(RefCell::new(Vec::new())) };
            match name {
                Some(name) => scope.assign(name, func.clone()),
                None => (),
            };
            Ok(func)
        },
        Expr::Struct { body } => unimplemented!(),
    }
}

fn match_pattern<'a, 'b>(pattern: &'b Parsed<'a, Expr<'a>>, ty: Type<'a, 'b>, scope: &mut Scope<'a, 'b>) -> Result<(), CompileError<'a>> {
    match pattern.get_node() {
        Expr::Ident(source) => {
            scope.assign(source, ty);
            Ok(())
        }
        Expr::Tuple { exprs } => match ty {
            Type::Tuple(types) if types.len() == exprs.len() => {
                for (ty, pattern) in types.iter().zip(exprs) {
                    match_pattern(pattern, ty.clone(), scope)?;
                }
                Ok(())
            }
            _ => Err(CompileError::type_error(pattern.get_source()))
        },
        _ => unimplemented!(),
    }
}