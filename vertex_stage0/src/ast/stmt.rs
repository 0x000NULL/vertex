use crate::ast::{Expr, Item, NodeId, Pattern, Type};
use crate::span::Span;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Stmt {
    Let {
        pattern: Pattern,
        ty: Option<Type>,
        init: Option<Expr>,
        span: Span,
        id: NodeId,
    },
    Expr {
        expr: Expr,
        has_semi: bool,
    },
    Item(Item),
}
