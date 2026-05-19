//! Lexical analysis for AQL.

use logos::Logos;
use std::fmt;

#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\r\n]+")]
#[logos(skip r"//[^\n]*")]
#[logos(skip r"/\*([^*]|\*+[^*/])*\*+/")]
pub enum TokenKind {
    #[token("|>")]
    Pipe,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token(",")]
    Comma,
    #[token("=")]
    Eq,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,
    #[token(">")]
    Gt,
    #[token("<")]
    Lt,
    #[token(">=")]
    Ge,
    #[token("<=")]
    Le,
    #[token("==")]
    EqEq,
    #[token("!=")]
    Ne,
    #[token("&&")]
    AndAnd,
    #[token("||")]
    OrOr,
    #[token(".")]
    Dot,
    #[token("source")]
    Source,
    #[token("sink")]
    Sink,
    #[token("filter")]
    Filter,
    #[token("map")]
    Map,
    #[token("flatMap")]
    FlatMap,
    #[token("keyBy")]
    KeyBy,
    #[token("window")]
    Window,
    #[token("aggregate")]
    Aggregate,
    #[token("watermark")]
    Watermark,
    #[token("join")]
    Join,
    #[token("union")]
    Union,
    #[token("split")]
    Split,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("match")]
    Match,
    #[token("let")]
    Let,
    #[token("in")]
    In,
    #[token("=>")]
    FatArrow,
    #[token("tumbling")]
    Tumbling,
    #[token("sliding")]
    Sliding,
    #[token("session")]
    Session,
    #[token("inner")]
    Inner,
    #[token("left")]
    Left,
    #[token("right")]
    Right,
    #[token("size")]
    Size,
    #[token("slide")]
    Slide,
    #[token("gap")]
    Gap,
    #[token("delay")]
    Delay,
    #[token("sum")]
    Sum,
    #[token("count")]
    Count,
    #[token("avg")]
    Avg,
    #[token("min")]
    Min,
    #[token("max")]
    Max,
    #[token("stddev")]
    Stddev,
    #[token("percentile")]
    Percentile,
    #[token("first")]
    First,
    #[token("last")]
    Last,
    #[regex(r#""([^"\\]|\\.)*""#)]
    StringLit,
    #[regex(r"[0-9]+(\.[0-9]+)?[smhd]?", |lex| lex.slice().to_string())]
    NumberOrDuration,
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Ident,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: (usize, usize),
}

pub struct Lexer<'a> {
    inner: logos::Lexer<'a, TokenKind>,
    source: &'a str,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            inner: TokenKind::lexer(source),
            source,
        }
    }

    pub fn tokenize(mut self) -> Result<Vec<Token>, String> {
        let mut out = Vec::new();
        while let Some(kind) = self.inner.next() {
            let kind = kind.map_err(|e| format!("lex error: {e:?}"))?;
            let span = self.inner.span();
            out.push(Token { kind, span });
        }
        Ok(out)
    }

    pub fn slice(&self, token: &Token) -> &str {
        &self.source[token.span.0..token.span.1]
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
