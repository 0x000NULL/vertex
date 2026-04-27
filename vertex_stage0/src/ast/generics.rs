use crate::ast::expr::{GenericArg, Path};
use crate::ast::{NodeId, Type};
use crate::span::Span;

#[derive(Debug, Clone)]
pub struct Generics {
    pub id: NodeId,
    pub span: Span,
    pub params: Vec<TypeParam>,
    pub where_clause: Option<WhereClause>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TypeParam {
    pub id: NodeId,
    pub span: Span,
    pub name: String,
    pub bounds: Vec<TraitBound>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WhereClause {
    pub id: NodeId,
    pub span: Span,
    pub predicates: Vec<WherePred>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WherePred {
    pub id: NodeId,
    pub span: Span,
    pub ty: Type,
    pub bounds: Vec<TraitBound>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TraitBound {
    pub id: NodeId,
    pub span: Span,
    pub path: Path,
    pub generic_args: Vec<GenericArg>,
}
