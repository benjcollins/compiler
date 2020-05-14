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
    Top,
    Block,
    Tuple,
    Bottom,
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
    pos.next_while(|ch| ch.is_whitespace())
}

fn parse_block<'a>(start: Position<'a>) -> Result<Parsed<'a, Expr<'a>>, ParseError<'a>> {
    let expr = match start.next() {
        Some((pos, '{')) => parse(skip_spaces(pos), Prec::Top),
        _ => Err(ParseError::expected_string(start, "{"))
    }?;
    match skip_spaces(expr.end()).next() {
        Some((end, '}')) => Ok(Parsed::new(start, end, expr.node)),
        _ => Err(ParseError::expected_string(skip_spaces(expr.end()), "}"))
    }
}

fn parse<'a>(start: Position<'a>, prec: Prec) -> Result<Parsed<'a, Expr<'a>>, ParseError> {
    let mut left = match start.next() {
        Some((pos, ch)) if ch.is_numeric() => {
            let end = pos.next_while(|ch| ch.is_numeric());
            Ok(Parsed::new(start, end, Expr::IntLiteral(Position::slice(start, end))))
        }
        Some((pos, '(')) => {
            let expr = parse(skip_spaces(pos), Prec::Block)?;
            match expr.end().next() {
                Some((end, ')')) => Ok(Parsed::new(start, end, expr.node)),
                _ => Err(ParseError::expected_string(skip_spaces(expr.end()), ")"))
            }
        }
        Some((pos, ch)) if ch.is_alphabetic() => {
            let end = pos.next_while(|ch| ch.is_alphanumeric());
            match Position::slice(start, end) {
                "fn" => {
                    let name = match skip_spaces(end).next() {
                        Some((pos, ch)) if ch.is_alphabetic() => {
                            let end_name = pos.next_while(|ch| ch.is_alphanumeric());
                            Parsed::new(skip_spaces(end), end_name, Some(Position::slice(skip_spaces(end), end_name)))
                        }
                        _ => Parsed::new(skip_spaces(end), skip_spaces(end), None),
                    };
                    let pattern = match skip_spaces(name.end()).next() {
                        Some((_, '(')) => parse(skip_spaces(name.end()), Prec::Tuple),
                        _ => Err(ParseError::expected_string(skip_spaces(end), "(")),
                    }?;
                    let expr = parse_block(skip_spaces(pattern.end()))?;
                    Ok(Parsed::new(start, expr.end(), Expr::Func { name: name.node, pattern: Box::new(pattern), expr: Box::new(expr) }))
                }
                "if" => {
                    let cond = parse(skip_spaces(end), Prec::Tuple)?;
                    let conc = parse_block(skip_spaces(cond.end()))?;
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
            Some((pos, '+')) if prec < Prec::Bottom => {
                Expr::new_binary(left, parse(skip_spaces(pos), prec)?, BinaryOp::Plus)
            }
            Some((pos, '=')) if prec < Prec::Bottom => {
                Expr::new_binary(left, parse(skip_spaces(pos), Prec::Bottom)?, BinaryOp::SingleEquals)
            }
            Some((_, '(')) if prec < Prec::Bottom => {
                Expr::new_binary(left, parse(start, Prec::Bottom)?, BinaryOp::Bracket)
            }
            Some((pos, ',')) if prec < Prec::Tuple => {
                Expr::new_tuple(left, parse(skip_spaces(pos), Prec::Tuple)?)
            }
            Some((_, ch)) if ch.is_alphabetic() => {
                let end = start.next_while(|ch| ch.is_alphabetic());
                let keyword = Position::slice(start, end);
                match keyword {
                    "else" => Expr::new_binary(left, parse_block(skip_spaces(end))?, BinaryOp::Else),
                    _ => if prec < Prec::Block && ch != '}' {
                        Expr::new_block(left, parse(start, Prec::Block)?)
                    } else {
                        return Ok(left)
                    }
                }
            }
            _ => return Ok(left)
        }
    }
}

pub fn parse_source(source: &str) -> Result<Parsed<Expr>, ParseError> {
    parse(Position::from_source(source), Prec::Top)
}