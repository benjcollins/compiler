use crate::position::Position;
use crate::ast::{Expr, Parsed, BinaryOp};

#[derive(Debug)]
pub struct ParseError<'a> {
    pos: Position<'a>,
    ty: ParseErrorType,
}

#[derive(Debug)]
pub enum ParseErrorType {
    ExpectedValue,
    ExpectedString(&'static str)
}

#[derive(PartialEq, PartialOrd, Copy, Clone)]
enum Prec {
    Block,
    Tuple,
    Expr,
    Sum,
}

impl<'a> ParseError<'a> {
    fn expected_value(pos: Position<'a>) -> ParseError<'a> {
        ParseError { pos, ty: ParseErrorType::ExpectedValue }
    }
    fn expected_string(pos: Position<'a>, string: &'static str) -> ParseError<'a> {
        ParseError { pos, ty: ParseErrorType::ExpectedString(string) }
    }
}

fn skip_spaces(pos: Position) -> Position {
    pos.next_while(|ch| ch.is_whitespace() && ch != '\n')
}

fn skip_lines(pos: Position) -> Position {
    pos.next_while(|ch| ch.is_whitespace())
}

fn parse<'a>(start: Position<'a>, prec: Prec) -> Result<Parsed<'a, Expr<'a>>, ParseError> {
    let mut left = match start.next() {
        Some((pos, ch)) if ch.is_numeric() => {
            let end = pos.next_while(|ch| ch.is_numeric());
            Ok(Parsed::new(start, end, Expr::IntLiteral(Position::slice(start, end))))
        }
        Some((pos, '(')) => {
            let expr = parse(skip_lines(pos), Prec::Tuple)?;
            match expr.end().next() {
                Some((end, ')')) => Ok(Parsed::new(start, end, expr.node)),
                _ => Err(ParseError::expected_string(skip_lines(expr.end()), ")"))
            }
        }
        Some((pos, '{')) => {
            let expr = parse(skip_lines(pos), Prec::Block)?;
            match skip_lines(expr.end()).next() {
                Some((end, '}')) => Ok(Parsed::new(start, end, expr.node)),
                _ => Err(ParseError::expected_string(skip_lines(expr.end()), "}"))
            }
        }
        Some((pos, ch)) if ch.is_alphabetic() => {
            let end = pos.next_while(|ch| ch.is_alphanumeric());
            match Position::slice(start, end) {
                "fn" => {
                    let name = match skip_lines(end).next() {
                        Some((pos, ch)) if ch.is_alphabetic() => {
                            let end_name = pos.next_while(|ch| ch.is_alphanumeric());
                            Parsed::new(skip_lines(end), end_name, Some(Position::slice(skip_lines(end), end_name)))
                        }
                        _ => Parsed::new(skip_lines(end), skip_lines(end), None),
                    };
                    let pattern = match skip_lines(name.end()).next() {
                        Some((_, '(')) => parse(skip_lines(name.end()), Prec::Expr),
                        _ => Err(ParseError::expected_string(skip_lines(end), "(")),
                    }?;
                    let expr = parse(skip_lines(pattern.end()), Prec::Expr)?;
                    Ok(Parsed::new(start, expr.end(), Expr::Func { name: name.node, pattern: Box::new(pattern), expr: Box::new(expr) }))
                }
                "struct" => {
                    let body = match skip_lines(end).next() {
                        Some((_, '{')) => parse(skip_lines(end), Prec::Expr),
                        _ => Err(ParseError::expected_string(skip_lines(end), "{")),
                    }?;
                    Ok(Parsed::new(start, body.end(), Expr::Struct { body: Box::new(body) }))
                }
                "if" => {
                    let cond = match skip_lines(end).next() {
                        Some((_, '(')) => parse(skip_lines(end), Prec::Expr),
                        _ => Err(ParseError::expected_string(skip_lines(end), "(")),
                    }?;
                    let conc = parse(skip_lines(cond.end()), Prec::Expr)?;
                    Ok(Parsed::new(start, conc.end(), Expr::If { cond: Box::new(cond), conc: Box::new(conc) }))
                }
                "true" | "false" => Ok(Parsed::new(start, end, Expr::BoolLiteral(Position::slice(start, end)))),
                _ => Ok(Parsed::new(start, end, Expr::Ident(Position::slice(start, end)))),
            }
        }
        _ => Err(ParseError::expected_value(start))
    }?;

    loop {
        let start = skip_spaces(left.end());
        left = match start.next() {
            Some((pos, '+')) if prec < Prec::Sum => {
                Expr::new_binary(left, parse(skip_lines(pos), prec)?, BinaryOp::Plus)
            }
            Some((pos, '=')) if prec <= Prec::Expr => {
                Expr::new_binary(left, parse(skip_lines(pos), Prec::Expr)?, BinaryOp::SingleEquals)
            }
            Some((_, '(')) => {
                Expr::new_binary(left, parse(start, Prec::Tuple)?, BinaryOp::Bracket)
            }
            Some((pos, ',')) if prec <= Prec::Tuple => {
                Expr::new_tuple(left, parse(skip_lines(pos), Prec::Tuple)?)
            }
            Some((_, ch)) if ch.is_alphabetic() => {
                let end = start.next_while(|ch| ch.is_alphabetic());
                let keyword = Position::slice(start, end);
                match keyword {
                    "else" => Expr::new_binary(left, parse(skip_lines(end), Prec::Expr)?, BinaryOp::Else),
                    _ => return Ok(left)
                }
            }
            _ => {
                let pos = skip_lines(start);
                match pos.next() {
                    Some((_, ch)) if ch != '}' && prec <= Prec::Block => Expr::new_block(left, parse(pos, Prec::Expr)?),
                    _ => return Ok(left),
                }
            }
        }
    }
}

pub fn parse_source(source: &str) -> Result<Parsed<Expr>, ParseError> {
    parse(Position::from_source(source), Prec::Block)
}