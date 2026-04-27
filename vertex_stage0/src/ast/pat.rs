use crate::ast::expr::{BoolLit, CharLit, FloatLit, IntLit, Path, StrLit};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Lit {
    Int(IntLit),
    Float(FloatLit),
    Char(CharLit),
    Str(StrLit),
    Bool(BoolLit),
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct StructPatField {
    pub name: String,
    pub pattern: Pattern,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Pattern {
    Wild,
    Ident {
        name: String,
        mutable: bool,
        sub: Option<Box<Pattern>>,
    },
    Literal(Lit),
    Range {
        start: Box<Pattern>,
        end: Box<Pattern>,
        inclusive: bool,
    },
    Tuple(Vec<Pattern>),
    Struct {
        path: Path,
        fields: Vec<StructPatField>,
        rest: bool,
    },
    TupleStruct {
        path: Path,
        elems: Vec<Pattern>,
    },
    Ref {
        mutable: bool,
        pattern: Box<Pattern>,
    },
    Or(Vec<Pattern>),
}
