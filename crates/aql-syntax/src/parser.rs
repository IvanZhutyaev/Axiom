//! Recursive-descent parser for AQL pipelines.

use crate::ast::*;
use crate::lexer::{Lexer, Token, TokenKind};
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum ParseError {
    #[error("unexpected token {0} at {1}")]
    Unexpected(String, usize),
    #[error("expected {expected} at {pos}")]
    Expected { expected: String, pos: usize },
    #[error("lex error: {0}")]
    Lex(String),
}

pub fn parse(source: &str) -> Result<Program, ParseError> {
    let tokens = Lexer::new(source)
        .tokenize()
        .map_err(ParseError::Lex)?;
    let mut p = Parser {
        source,
        tokens,
        pos: 0,
    };
    p.parse_program()
}

struct Parser<'a> {
    source: &'a str,
    tokens: Vec<Token>,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Option<&TokenKind> {
        self.tokens.get(self.pos).map(|t| &t.kind)
    }

    fn bump(&mut self) -> Option<Token> {
        if self.pos < self.tokens.len() {
            let t = self.tokens[self.pos].clone();
            self.pos += 1;
            Some(t)
        } else {
            None
        }
    }

    fn expect_kind(&mut self, kind: TokenKind) -> Result<(), ParseError> {
        match self.peek() {
            Some(k) if *k == kind => {
                self.bump();
                Ok(())
            }
            _ => Err(ParseError::Expected {
                expected: format!("{kind:?}"),
                pos: self.pos,
            }),
        }
    }

    fn slice(&self, token: &Token) -> &str {
        &self.source[token.span.0..token.span.1]
    }

    fn parse_program(&mut self) -> Result<Program, ParseError> {
        let mut stages = Vec::new();
        stages.push(self.parse_first_stage()?);
        while self.peek() == Some(&TokenKind::Pipe) {
            self.bump();
            stages.push(self.parse_stage()?);
        }
        Ok(Program { stages })
    }

    fn parse_first_stage(&mut self) -> Result<Stage, ParseError> {
        self.parse_stage()
    }

    fn parse_stage(&mut self) -> Result<Stage, ParseError> {
        match self.peek() {
            Some(TokenKind::Source) => {
                self.bump();
                let name = self.parse_string_lit()?;
                Ok(Stage::Source { name })
            }
            Some(TokenKind::Sink) => {
                self.bump();
                let name = self.parse_string_lit()?;
                Ok(Stage::Sink { name })
            }
            Some(TokenKind::Filter) => {
                self.bump();
                self.expect_kind(TokenKind::LParen)?;
                let predicate = self.parse_expr(0)?;
                self.expect_kind(TokenKind::RParen)?;
                Ok(Stage::Filter { predicate })
            }
            Some(TokenKind::Map) => {
                self.bump();
                self.expect_kind(TokenKind::LParen)?;
                let projection = self.parse_field_bindings()?;
                self.expect_kind(TokenKind::RParen)?;
                Ok(Stage::Map { projection })
            }
            Some(TokenKind::FlatMap) => {
                self.bump();
                self.expect_kind(TokenKind::LParen)?;
                let expr = self.parse_expr(0)?;
                self.expect_kind(TokenKind::RParen)?;
                Ok(Stage::FlatMap { expr })
            }
            Some(TokenKind::KeyBy) => {
                self.bump();
                self.expect_kind(TokenKind::LParen)?;
                let key = self.parse_expr(0)?;
                self.expect_kind(TokenKind::RParen)?;
                Ok(Stage::KeyBy { key })
            }
            Some(TokenKind::Window) => {
                self.parse_window()
            }
            Some(TokenKind::Watermark) => {
                self.bump();
                self.expect_kind(TokenKind::LParen)?;
                let field = self.parse_ident()?;
                let mut delay_ms = 5000;
                if self.peek() == Some(&TokenKind::Comma) {
                    self.bump();
                    self.expect_kind(TokenKind::Delay)?;
                    self.expect_kind(TokenKind::Eq)?;
                    delay_ms = self.parse_duration()?;
                }
                self.expect_kind(TokenKind::RParen)?;
                Ok(Stage::Watermark { field, delay_ms })
            }
            Some(TokenKind::Join) => {
                self.parse_join()
            }
            Some(TokenKind::Union) => {
                self.bump();
                self.expect_kind(TokenKind::LParen)?;
                let mut streams = vec![self.parse_string_lit()?];
                while self.peek() == Some(&TokenKind::Comma) {
                    self.bump();
                    streams.push(self.parse_string_lit()?);
                }
                self.expect_kind(TokenKind::RParen)?;
                Ok(Stage::Union { streams })
            }
            Some(TokenKind::Split) => {
                self.bump();
                self.expect_kind(TokenKind::LParen)?;
                let mut branches = vec![self.parse_string_lit()?];
                while self.peek() == Some(&TokenKind::Comma) {
                    self.bump();
                    branches.push(self.parse_string_lit()?);
                }
                self.expect_kind(TokenKind::RParen)?;
                Ok(Stage::Split { branches })
            }
            other => Err(ParseError::Unexpected(
                format!("{other:?}"),
                self.pos,
            )),
        }
    }

    fn parse_window(&mut self) -> Result<Stage, ParseError> {
        self.bump();
        self.expect_kind(TokenKind::LParen)?;
        let kind = match self.peek() {
            Some(TokenKind::Tumbling) => {
                self.bump();
                WindowKind::Tumbling
            }
            Some(TokenKind::Sliding) => {
                self.bump();
                WindowKind::Sliding
            }
            Some(TokenKind::Session) => {
                self.bump();
                WindowKind::Session
            }
            _ => {
                return Err(ParseError::Expected {
                    expected: "window kind".into(),
                    pos: self.pos,
                });
            }
        };
        self.expect_kind(TokenKind::Comma)?;
        let mut size_ms = 0u64;
        let mut slide_ms = None;
        let mut gap_ms = None;
        loop {
            match self.peek() {
                Some(TokenKind::Size) => {
                    self.bump();
                    self.expect_kind(TokenKind::Eq)?;
                    size_ms = self.parse_duration()?;
                }
                Some(TokenKind::Slide) => {
                    self.bump();
                    self.expect_kind(TokenKind::Eq)?;
                    slide_ms = Some(self.parse_duration()?);
                }
                Some(TokenKind::Gap) => {
                    self.bump();
                    self.expect_kind(TokenKind::Eq)?;
                    gap_ms = Some(self.parse_duration()?);
                }
                _ => break,
            }
            if self.peek() == Some(&TokenKind::Comma) {
                self.bump();
            }
        }
        self.expect_kind(TokenKind::RParen)?;
        let mut aggregates = Vec::new();
        if self.peek() == Some(&TokenKind::Aggregate) {
            self.bump();
            self.expect_kind(TokenKind::LParen)?;
            aggregates = self.parse_aggregates()?;
            self.expect_kind(TokenKind::RParen)?;
        }
        Ok(Stage::Window {
            kind,
            size_ms,
            slide_ms,
            gap_ms,
            aggregates,
        })
    }

    fn parse_join(&mut self) -> Result<Stage, ParseError> {
        self.bump();
        self.expect_kind(TokenKind::LParen)?;
        let other = self.parse_string_lit()?;
        self.expect_kind(TokenKind::Comma)?;
        let join_type = match self.peek() {
            Some(TokenKind::Inner) => {
                self.bump();
                JoinType::Inner
            }
            Some(TokenKind::Left) => {
                self.bump();
                JoinType::Left
            }
            Some(TokenKind::Right) => {
                self.bump();
                JoinType::Right
            }
            _ => JoinType::Inner,
        };
        self.expect_kind(TokenKind::Comma)?;
        let key = self.parse_expr(0)?;
        let mut window_ms = 0;
        if self.peek() == Some(&TokenKind::Comma) {
            self.bump();
            window_ms = self.parse_duration()?;
        }
        self.expect_kind(TokenKind::RParen)?;
        Ok(Stage::Join {
            other,
            join_type,
            key,
            window_ms,
        })
    }

    fn parse_aggregates(&mut self) -> Result<Vec<Aggregate>, ParseError> {
        let mut aggs = Vec::new();
        loop {
            let name = self.parse_ident()?;
            self.expect_kind(TokenKind::Eq)?;
            let (func, arg) = self.parse_agg_func()?;
            aggs.push(Aggregate { name, func, arg });
            if self.peek() != Some(&TokenKind::Comma) {
                break;
            }
            self.bump();
        }
        Ok(aggs)
    }

    fn parse_agg_func(&mut self) -> Result<(AggFunc, Option<Expr>), ParseError> {
        match self.peek() {
            Some(TokenKind::Sum) => {
                self.bump();
                self.expect_kind(TokenKind::LParen)?;
                let arg = Some(self.parse_expr(0)?);
                self.expect_kind(TokenKind::RParen)?;
                Ok((AggFunc::Sum, arg))
            }
            Some(TokenKind::Count) => {
                self.bump();
                self.expect_kind(TokenKind::LParen)?;
                let arg = if self.peek() == Some(&TokenKind::Star) {
                    self.bump();
                    None
                } else {
                    Some(self.parse_expr(0)?)
                };
                self.expect_kind(TokenKind::RParen)?;
                Ok((AggFunc::Count, arg))
            }
            Some(TokenKind::Avg) => {
                self.bump();
                self.expect_kind(TokenKind::LParen)?;
                let arg = Some(self.parse_expr(0)?);
                self.expect_kind(TokenKind::RParen)?;
                Ok((AggFunc::Avg, arg))
            }
            Some(TokenKind::Min) => {
                self.bump();
                self.expect_kind(TokenKind::LParen)?;
                let arg = Some(self.parse_expr(0)?);
                self.expect_kind(TokenKind::RParen)?;
                Ok((AggFunc::Min, arg))
            }
            Some(TokenKind::Max) => {
                self.bump();
                self.expect_kind(TokenKind::LParen)?;
                let arg = Some(self.parse_expr(0)?);
                self.expect_kind(TokenKind::RParen)?;
                Ok((AggFunc::Max, arg))
            }
            Some(TokenKind::Stddev) => {
                self.bump();
                self.expect_kind(TokenKind::LParen)?;
                let arg = Some(self.parse_expr(0)?);
                self.expect_kind(TokenKind::RParen)?;
                Ok((AggFunc::Stddev, arg))
            }
            Some(TokenKind::Percentile) => {
                self.bump();
                self.expect_kind(TokenKind::LParen)?;
                let lit = self.parse_number()?;
                let p = match lit {
                    Literal::Float(f) => f,
                    Literal::Int(i) => i as f64,
                    _ => 0.5,
                };
                self.expect_kind(TokenKind::Comma)?;
                let arg = Some(self.parse_expr(0)?);
                self.expect_kind(TokenKind::RParen)?;
                Ok((AggFunc::Percentile(p), arg))
            }
            Some(TokenKind::First) => {
                self.bump();
                self.expect_kind(TokenKind::LParen)?;
                let arg = Some(self.parse_expr(0)?);
                self.expect_kind(TokenKind::RParen)?;
                Ok((AggFunc::First, arg))
            }
            Some(TokenKind::Last) => {
                self.bump();
                self.expect_kind(TokenKind::LParen)?;
                let arg = Some(self.parse_expr(0)?);
                self.expect_kind(TokenKind::RParen)?;
                Ok((AggFunc::Last, arg))
            }
            _ => Err(ParseError::Expected {
                expected: "aggregate function".into(),
                pos: self.pos,
            }),
        }
    }

    fn parse_field_bindings(&mut self) -> Result<Vec<FieldBinding>, ParseError> {
        let mut bindings = Vec::new();
        loop {
            let name = self.parse_ident()?;
            self.expect_kind(TokenKind::Eq)?;
            let expr = self.parse_expr(0)?;
            bindings.push(FieldBinding { name, expr });
            if self.peek() != Some(&TokenKind::Comma) {
                break;
            }
            self.bump();
        }
        Ok(bindings)
    }

    fn parse_string_lit(&mut self) -> Result<String, ParseError> {
        let t = self.bump().ok_or(ParseError::Expected {
            expected: "string".into(),
            pos: self.pos,
        })?;
        if t.kind != TokenKind::StringLit {
            return Err(ParseError::Expected {
                expected: "string literal".into(),
                pos: self.pos,
            });
        }
        let s = self.slice(&t);
        Ok(unescape_string(&s[1..s.len() - 1]))
    }

    fn parse_ident(&mut self) -> Result<String, ParseError> {
        let t = self.bump().ok_or(ParseError::Expected {
            expected: "ident".into(),
            pos: self.pos,
        })?;
        match t.kind {
            TokenKind::Ident => Ok(self.slice(&t).to_string()),
            _ => Err(ParseError::Expected {
                expected: "identifier".into(),
                pos: self.pos,
            }),
        }
    }

    fn parse_duration(&mut self) -> Result<u64, ParseError> {
        let t = self.bump().ok_or(ParseError::Expected {
            expected: "duration".into(),
            pos: self.pos,
        })?;
        let s = self.slice(&t);
        parse_duration_str(s).ok_or(ParseError::Expected {
            expected: "duration".into(),
            pos: self.pos,
        })
    }

    fn parse_number(&mut self) -> Result<Literal, ParseError> {
        let t = self.bump().ok_or(ParseError::Expected {
            expected: "number".into(),
            pos: self.pos,
        })?;
        let s = self.slice(&t);
        if let Ok(i) = s.parse::<i64>() {
            return Ok(Literal::Int(i));
        }
        if let Ok(f) = s.parse::<f64>() {
            return Ok(Literal::Float(f));
        }
        Err(ParseError::Expected {
            expected: "number".into(),
            pos: self.pos,
        })
    }

    fn parse_expr(&mut self, min_bp: u8) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_prefix()?;
        loop {
            let op = match self.peek() {
                Some(TokenKind::Plus) => Some((BinOp::Add, 10)),
                Some(TokenKind::Minus) => Some((BinOp::Sub, 10)),
                Some(TokenKind::Star) => Some((BinOp::Mul, 20)),
                Some(TokenKind::Slash) => Some((BinOp::Div, 20)),
                Some(TokenKind::Percent) => Some((BinOp::Mod, 20)),
                Some(TokenKind::EqEq) => Some((BinOp::Eq, 5)),
                Some(TokenKind::Ne) => Some((BinOp::Ne, 5)),
                Some(TokenKind::Lt) => Some((BinOp::Lt, 5)),
                Some(TokenKind::Gt) => Some((BinOp::Gt, 5)),
                Some(TokenKind::Le) => Some((BinOp::Le, 5)),
                Some(TokenKind::Ge) => Some((BinOp::Ge, 5)),
                Some(TokenKind::AndAnd) => Some((BinOp::And, 3)),
                Some(TokenKind::OrOr) => Some((BinOp::Or, 2)),
                Some(TokenKind::Dot) => {
                    self.bump();
                    let field = self.parse_ident()?;
                    lhs = Expr::Field(Box::new(lhs), field);
                    continue;
                }
                _ => None,
            };
            let Some((op, l_bp)) = op else { break };
            if l_bp < min_bp {
                break;
            }
            self.bump();
            let rhs = self.parse_expr(l_bp + 1)?;
            lhs = Expr::Binary {
                op,
                left: Box::new(lhs),
                right: Box::new(rhs),
            };
        }
        Ok(lhs)
    }

    fn parse_prefix(&mut self) -> Result<Expr, ParseError> {
        match self.peek() {
            Some(TokenKind::Minus) => {
                self.bump();
                Ok(Expr::Unary {
                    op: UnOp::Neg,
                    expr: Box::new(self.parse_expr(30)?),
                })
            }
            Some(TokenKind::LParen) => {
                self.bump();
                let e = self.parse_expr(0)?;
                self.expect_kind(TokenKind::RParen)?;
                Ok(e)
            }
            Some(TokenKind::StringLit) => {
                let s = self.parse_string_lit()?;
                Ok(Expr::Literal(Literal::String(s)))
            }
            Some(TokenKind::NumberOrDuration) | Some(TokenKind::Ident) => {
                let t = self.bump().unwrap();
                let s = self.slice(&t);
                if let Ok(i) = s.parse::<i64>() {
                    return Ok(Expr::Literal(Literal::Int(i)));
                }
                if let Ok(f) = s.parse::<f64>() {
                    return Ok(Expr::Literal(Literal::Float(f)));
                }
                if t.kind == TokenKind::Ident {
                    if self.peek() == Some(&TokenKind::LParen) {
                        self.bump();
                        let mut args = Vec::new();
                        if self.peek() != Some(&TokenKind::RParen) {
                            args.push(self.parse_expr(0)?);
                            while self.peek() == Some(&TokenKind::Comma) {
                                self.bump();
                                args.push(self.parse_expr(0)?);
                            }
                        }
                        self.expect_kind(TokenKind::RParen)?;
                        return Ok(Expr::Call {
                            name: s.to_string(),
                            args,
                        });
                    }
                    return Ok(Expr::Ident(s.to_string()));
                }
                Err(ParseError::Unexpected(s.to_string(), self.pos))
            }
            _ => Err(ParseError::Expected {
                expected: "expression".into(),
                pos: self.pos,
            }),
        }
    }
}

fn unescape_string(s: &str) -> String {
    s.replace("\\\"", "\"")
        .replace("\\\\", "\\")
        .replace("\\n", "\n")
}

fn parse_duration_str(s: &str) -> Option<u64> {
    if let Some(num) = s.strip_suffix('s') {
        return num.parse::<u64>().ok().map(|n| n * 1000);
    }
    if let Some(num) = s.strip_suffix('m') {
        return num.parse::<u64>().ok().map(|n| n * 60_000);
    }
    if let Some(num) = s.strip_suffix('h') {
        return num.parse::<u64>().ok().map(|n| n * 3_600_000);
    }
    if let Some(num) = s.strip_suffix('d') {
        return num.parse::<u64>().ok().map(|n| n * 86_400_000);
    }
    s.parse::<u64>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Stage;

    const TZ_EXAMPLE: &str = r#"source "sensor_data"
|> filter(temperature > 30.0)
|> window(tumbling, size=5s)
   aggregate(avg_temp = avg(temperature), count = count(*))
|> sink "alerts""#;

    #[test]
    fn parse_tz_example() {
        let prog = parse(TZ_EXAMPLE).expect("parse tz example");
        assert_eq!(prog.stages.len(), 4);
        assert!(matches!(prog.stages[0], Stage::Source { .. }));
        assert!(matches!(prog.stages[1], Stage::Filter { .. }));
        assert!(matches!(prog.stages[2], Stage::Window { .. }));
        assert!(matches!(prog.stages[3], Stage::Sink { .. }));
    }
}
