use crate::ast::{Parsed, Expr, BinaryOp};
use crate::{scope::Scope, ir::{Program, Block, Function}, types::{Implementation, Type}};
use std::{cell::RefCell, rc::Rc, collections::HashMap};

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

pub fn call_function<'a, 'b>(imp: &Implementation<'a, 'b>, argument_ty: Type<'a, 'b>, function: &mut Function, block: &mut Block) -> Type<'a, 'b> {
    let returns = block.call(imp.function, argument_ty.get_used_vars(), imp.return_ty.get_used_vars().len(), function);
    let mut map = HashMap::new();
    for (i, var) in imp.return_ty.get_used_vars().iter().enumerate() {
        map.insert(*var, returns[i]);
    }
    let return_ty = imp.return_ty.map_to(&map);
    return_ty
}

pub fn compile<'a, 'b>(expr: &'b Parsed<'a, Expr<'a>>, scope: &mut Scope<'a, 'b>, program: &mut Program, function: &mut Function, block: &mut Block) -> Result<Type<'a, 'b>, CompileError<'a>> {
    match expr.get_node() {
        Expr::IntLiteral => {
            let value = expr.get_source().parse::<i32>().unwrap();
            Ok(Type::Int(block.constant_int(value, function)))
        }
        Expr::Binary { left, right, op } => match op {
            BinaryOp::Plus => match (compile(left, scope, program, function, block)?, compile(right, scope, program, function, block)?) {
                (Type::Int(a), Type::Int(b)) => Ok(Type::Int(block.add_int(a, b, function))),
                _ => Err(CompileError::type_error(expr.get_source()))
            }
            BinaryOp::Bracket => {
                match compile(left, scope, program, function, block)? {
                    Type::Func { pattern, expr, impls } => {
                        let argument_ty = compile(right, scope, program, function, block)?;
                        for imp in impls.borrow().iter() {
                            if imp.param_ty == argument_ty {
                                return Ok(call_function(imp, argument_ty, function, block))
                            }
                        }
                        let mut new_function = Function::new();
                        let mut new_block = new_function.new_block();
                        let param_ty = argument_ty.as_parameter_ty(&mut new_function);
                        let mut function_scope = Scope::new();
                        match_pattern(pattern, param_ty.clone(), &mut function_scope)?;
                        let return_ty = compile(expr, &mut function_scope, program, &mut new_function, &mut new_block)?;
                        return_ty.return_ty(&mut new_function);
                        new_block.ret();
                        new_function.submit_block(new_block);
                        let new_function_id = program.add_function(new_function);
                        let imp = Implementation { param_ty, return_ty: return_ty.clone(), function: new_function_id };
                        let return_ty = call_function(&imp, argument_ty, function, block);
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
            BinaryOp::Else => unimplemented!(),
        }
        Expr::If { cond, conc } => {
            if let Type::Bool(cond) = compile(cond, scope, program, function, block)? {
                let mut cond_block = function.new_block();
                let exit_block = function.new_block();
                let conc = compile(conc, scope, program, function, &mut cond_block)?;
                block.branch_if(cond, cond_block.get_id());
                cond_block.branch(exit_block.get_id());
                block.branch(exit_block.get_id());
                function.submit_block(block.clone());
                function.submit_block(cond_block);
                *block = exit_block;
                Ok(Type::Either(cond, vec![conc, Type::Nothing]))
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
        Expr::Ident => match scope.get(expr.get_source()) {
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
        Expr::BoolLiteral => {
            Ok(Type::Bool(block.constant_int(if expr.get_source() == "true" { 1 } else { 0 }, function)))
        }
    }
}

fn match_pattern<'a, 'b>(pattern: &'b Parsed<'a, Expr<'a>>, ty: Type<'a, 'b>, scope: &mut Scope<'a, 'b>) -> Result<(), CompileError<'a>> {
    match pattern.get_node() {
        Expr::Ident => {
            scope.assign(pattern.get_source(), ty);
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