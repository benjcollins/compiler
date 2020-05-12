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

pub fn compile<'a, 'b>(expr: &'b Parsed<'a, Expr<'a>>, scope: &mut Scope<'a, 'b>, program: &mut Program, function: &mut Function, block: &mut Block) -> Result<Type<'a, 'b>, CompileError<'a>> {
    match expr.get_node() {
        Expr::IntLiteral => {
            let value = expr.get_source().parse::<i32>().unwrap();
            Ok(Type::Int(block.constant_int(value, program)))
        }
        Expr::Binary { left, right, op } => match op {
            BinaryOp::Plus => match (compile(left, scope, program, function, block)?, compile(right, scope, program, function, block)?) {
                (Type::Int(a), Type::Int(b)) => Ok(Type::Int(block.add_int(a, b, program))),
                _ => Err(CompileError::type_error(expr.get_source()))
            }
            BinaryOp::Bracket => {
                match compile(left, scope, program, function, block)? {
                    Type::Func { pattern, expr, impls } => {
                        let args = compile(right, scope, program, function, block)?;
                        for imp in impls.borrow().iter() {
                            if imp.params == args {
                                block.call(imp.function, args.vars_as_vec());
                                return Ok(imp.returns.clone())
                            }
                        }
                        let mut new_function = Function::new();
                        let mut new_block = Block::new();
                        let params = args.as_paramater(program, &mut new_function);
                        let mut function_scope = Scope::new();
                        match_pattern(pattern, params.clone(), &mut function_scope)?;
                        let returns = compile(expr, &mut function_scope, program, &mut new_function, &mut new_block)?;
                        new_block.ret();
                        let new_block_id = new_function.add_block(new_block);
                        new_function.set_main(new_block_id);
                        let new_function_id = program.add_function(new_function);
                        block.call(new_function_id, args.vars_as_vec());
                        impls.borrow_mut().push(Implementation { params, returns: returns.clone(), function: new_function_id });
                        Ok(returns)
                    }
                    _ => Err(CompileError::type_error(expr.get_source()))
                }
            },
            BinaryOp::SingleEquals => {
                let ty = compile(right, scope, program, function, block)?;
                match_pattern(left, ty.clone(), scope)?;
                Ok(ty)
            }
        }
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
                Some(name) => scope.define(name, func.clone()),
                None => (),
            };
            Ok(func)
        },
    }
}

fn match_pattern<'a, 'b>(pattern: &'b Parsed<'a, Expr<'a>>, ty: Type<'a, 'b>, scope: &mut Scope<'a, 'b>) -> Result<(), CompileError<'a>> {
    match pattern.get_node() {
        Expr::Ident => {
            scope.define(pattern.get_source(), ty);
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