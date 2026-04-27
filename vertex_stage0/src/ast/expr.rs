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

// TODO: replaced by define-type-enum-in-src-ast-ty-rs
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum CastTy {
    Placeholder,
}

// TODO: replaced by define-pattern-enum-in-src-ast-pat-rs
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum ClosureParam {
    Placeholder,
}

// TODO: replaced by define-stmt-enum-in-src-ast-stmt-rs
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Stmt {
    Placeholder,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct StructLitField {
    pub name: String,
    pub value: Expr,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
    BitNot,
    Deref,
    Ref,
    RefMut,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    RemAssign,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Unary {
    pub id: NodeId,
    pub span: Span,
    pub op: UnaryOp,
    pub operand: Box<Expr>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Binary {
    pub id: NodeId,
    pub span: Span,
    pub op: BinaryOp,
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Call {
    pub id: NodeId,
    pub span: Span,
    pub callee: Box<Expr>,
    pub args: Vec<Expr>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MethodCall {
    pub id: NodeId,
    pub span: Span,
    pub receiver: Box<Expr>,
    pub method: String,
    pub args: Vec<Expr>,
    pub generic_args: Vec<GenericArg>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FieldAccess {
    pub id: NodeId,
    pub span: Span,
    pub receiver: Box<Expr>,
    pub name: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TupleFieldAccess {
    pub id: NodeId,
    pub span: Span,
    pub receiver: Box<Expr>,
    pub idx: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Index {
    pub id: NodeId,
    pub span: Span,
    pub receiver: Box<Expr>,
    pub idx: Box<Expr>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Cast {
    pub id: NodeId,
    pub span: Span,
    pub expr: Box<Expr>,
    pub ty: Box<CastTy>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Try {
    pub id: NodeId,
    pub span: Span,
    pub expr: Box<Expr>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Range {
    pub id: NodeId,
    pub span: Span,
    pub start: Option<Box<Expr>>,
    pub end: Option<Box<Expr>>,
    pub inclusive: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Closure {
    pub id: NodeId,
    pub span: Span,
    pub params: Vec<ClosureParam>,
    pub body: Box<Expr>,
    pub move_kw: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct StructLit {
    pub id: NodeId,
    pub span: Span,
    pub path: Path,
    pub fields: Vec<StructLitField>,
    pub base: Option<Box<Expr>>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TupleLit {
    pub id: NodeId,
    pub span: Span,
    pub elems: Vec<Expr>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ArrayLit {
    pub id: NodeId,
    pub span: Span,
    pub elems: Vec<Expr>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ArrayRepeat {
    pub id: NodeId,
    pub span: Span,
    pub value: Box<Expr>,
    pub count: Box<Expr>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Block {
    pub id: NodeId,
    pub span: Span,
    pub stmts: Vec<Stmt>,
    pub tail: Option<Box<Expr>>,
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
    Unary(Unary),
    Binary(Binary),
    Call(Call),
    MethodCall(MethodCall),
    Field(FieldAccess),
    TupleField(TupleFieldAccess),
    Index(Index),
    Cast(Cast),
    Try(Try),
    Range(Range),
    Closure(Closure),
    StructLit(StructLit),
    TupleLit(TupleLit),
    ArrayLit(ArrayLit),
    ArrayRepeat(ArrayRepeat),
    Block(Block),
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
            Expr::Unary(e) => e.id,
            Expr::Binary(e) => e.id,
            Expr::Call(e) => e.id,
            Expr::MethodCall(e) => e.id,
            Expr::Field(e) => e.id,
            Expr::TupleField(e) => e.id,
            Expr::Index(e) => e.id,
            Expr::Cast(e) => e.id,
            Expr::Try(e) => e.id,
            Expr::Range(e) => e.id,
            Expr::Closure(e) => e.id,
            Expr::StructLit(e) => e.id,
            Expr::TupleLit(e) => e.id,
            Expr::ArrayLit(e) => e.id,
            Expr::ArrayRepeat(e) => e.id,
            Expr::Block(e) => e.id,
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
            Expr::Unary(e) => e.span,
            Expr::Binary(e) => e.span,
            Expr::Call(e) => e.span,
            Expr::MethodCall(e) => e.span,
            Expr::Field(e) => e.span,
            Expr::TupleField(e) => e.span,
            Expr::Index(e) => e.span,
            Expr::Cast(e) => e.span,
            Expr::Try(e) => e.span,
            Expr::Range(e) => e.span,
            Expr::Closure(e) => e.span,
            Expr::StructLit(e) => e.span,
            Expr::TupleLit(e) => e.span,
            Expr::ArrayLit(e) => e.span,
            Expr::ArrayRepeat(e) => e.span,
            Expr::Block(e) => e.span,
        }
    }
}
