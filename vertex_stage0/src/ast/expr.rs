use crate::ast::NodeId;
use crate::lexer::token::{FloatSuffix, IntSuffix};
use crate::span::Span;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct IntLit {
    pub id: NodeId,
    pub span: Span,
    pub value: u64,
    pub suffix: IntSuffix,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FloatLit {
    pub id: NodeId,
    pub span: Span,
    pub value: f64,
    pub suffix: FloatSuffix,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CharLit {
    pub id: NodeId,
    pub span: Span,
    pub value: char,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct StrLit {
    pub id: NodeId,
    pub span: Span,
    pub value: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BoolLit {
    pub id: NodeId,
    pub span: Span,
    pub value: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Path {
    pub id: NodeId,
    pub span: Span,
    pub segments: Vec<PathSegment>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PathSegment {
    pub ident: String,
    pub generic_args: Vec<GenericArg>,
}

// TODO: replaced/merged by define-type-enum-in-src-ast-ty-rs and
// define-generics-and-whereclause-in-src-ast-generics-rs.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum GenericArg {
    Placeholder,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Expr {
    IntLit(IntLit),
    FloatLit(FloatLit),
    CharLit(CharLit),
    StrLit(StrLit),
    BoolLit(BoolLit),
    Path(Path),
}

impl Expr {
    pub fn id(&self) -> NodeId {
        match self {
            Expr::IntLit(e) => e.id,
            Expr::FloatLit(e) => e.id,
            Expr::CharLit(e) => e.id,
            Expr::StrLit(e) => e.id,
            Expr::BoolLit(e) => e.id,
            Expr::Path(e) => e.id,
        }
    }

    pub fn span(&self) -> Span {
        match self {
            Expr::IntLit(e) => e.span,
            Expr::FloatLit(e) => e.span,
            Expr::CharLit(e) => e.span,
            Expr::StrLit(e) => e.span,
            Expr::BoolLit(e) => e.span,
            Expr::Path(e) => e.span,
        }
    }
}
