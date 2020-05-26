use crate::ast::{Parsed, Expr, BinaryOp};
use crate::{scope::Scope, ir::{Program, Block, Function}, types::{Implementation, Type}};
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

pub fn call_function<'a, 'b>(imp: &Implementation<'a, 'b>, argument_ty: Type<'a, 'b>, program: &mut Program, block: &mut Block) -> Type<'a, 'b> {
    let returns = block.call(imp.function, argument_ty.get_used_vars(), program);
    imp.return_ty.map_to(&returns)
}

pub fn compile<'a, 'b>(expr: &'b Parsed<'a, Expr<'a>>, scope: &mut Scope<'a, 'b>, program: &mut Program, function: &mut Function, block: &mut Block) -> Result<Type<'a, 'b>, CompileError<'a>> {
    match expr.get_node() {
        Expr::IntLiteral(source) => {
            let value = source.parse::<i32>().unwrap();
            Ok(Type::Int(block.constant_int(value, program)))
        }
        Expr::Binary { left, right, op } => match op {
            BinaryOp::Plus => {
                let left = compile(left, scope, program, function, block)?;
                let right = compile(right, scope, program, function, block)?;
                match (left, right) {
                    (Type::Int(a), Type::Int(b)) => Ok(Type::Int(block.add_int(a, b, program))),
                    _ => Err(CompileError::type_error(expr.get_source()))
                }
            }
            BinaryOp::Bracket => {
                match compile(left, scope, program, function, block)? {
                    Type::Func { pattern, expr, impls } => {
                        let argument_ty = compile(right, scope, program, function, block)?;
                        for imp in impls.borrow().iter() {
                            if imp.param_ty == argument_ty {
                                return Ok(call_function(imp, argument_ty, program, block))
                            }
                        }
                        let mut new_function = Function::new();
                        let mut new_block = new_function.new_block();
                        let param_ty = argument_ty.as_parameter_ty(&mut new_function, program);
                        let mut function_scope = Scope::new();
                        match_pattern(pattern, param_ty.clone(), &mut function_scope)?;
                        let return_ty = compile(expr, &mut function_scope, program, &mut new_function, &mut new_block)?;
                        return_ty.return_ty(&mut new_function);
                        new_block.ret(&mut new_function);
                        let new_function_id = program.add_function(new_function);
                        let imp = Implementation { param_ty, return_ty: return_ty.clone(), function: new_function_id };
                        let return_ty = call_function(&imp, argument_ty, program, block);
                        impls.borrow_mut().push(imp);
                        Ok(return_ty)
                    }
                    _ => Err(CompileError::type_error(expr.get_source()))
                }
            },
            BinaryOp::SingleEquals => {
                let ty = compile(right, scope, program, function, block)?;
                match_pattern(left, ty.clone(), scope)?;
                Ok(ty)
            }
            BinaryOp::Else => {
                if let Type::Maybe(tag, ty) = compile(left, scope, program, function, block)? {
                    let mut cond_block = function.new_block();
                    let exit_block = function.new_block();
                    block.clone().conditional_branch(tag, exit_block.get_id(), cond_block.get_id(), function);
                    let conc = compile(right, scope, program, function, &mut cond_block)?;
                    cond_block.branch(exit_block.get_id(), function);
                    *block = exit_block;
                    Ok(Type::merge(tag, &*ty, &conc, program, block))
                } else {
                    Err(CompileError::type_error(expr.get_source()))
                }
            },
        }
        Expr::If { cond, conc } => {
            if let Type::Bool(cond) = compile(cond, scope, program, function, block)? {
                let mut cond_block = function.new_block();
                let exit_block = function.new_block();
                block.clone().conditional_branch(cond, cond_block.get_id(), exit_block.get_id(), function);
                let conc = compile(conc, scope, program, function, &mut cond_block)?;
                cond_block.branch(exit_block.get_id(), function);
                *block = exit_block;
                Ok(Type::Maybe(cond, Box::new(conc)))
            } else {
                Err(CompileError::type_error(expr.get_source()))
            }
        },
        Expr::Tuple { exprs } => {
            let mut types = Vec::new();
            for expr in exprs {
                types.push(compile(expr, scope, program, function, block)?)
            }
            Ok(Type::Tuple(types))
        }
        Expr::Block { exprs, last } => {
            for expr in exprs {
                compile(expr, scope, program, function, block)?;
            }
            compile(last, scope, program, function, block)
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
        Expr::BoolLiteral(source) => {
            Ok(Type::Bool(block.constant_int(if *source == "true" { 1 } else { 0 }, program)))
        }
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