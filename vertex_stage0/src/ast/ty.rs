use crate::ast::{expr::Path, NodeId};
use crate::span::Span;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Type {
    Path(Path),
    Ref {
        mutable: bool,
        ty: Box<Type>,
        span: Span,
        id: NodeId,
    },
    Ptr {
        mutable: bool,
        ty: Box<Type>,
    },
    Array {
        elem: Box<Type>,
        len: Box<crate::ast::expr::Expr>,
    },
    Slice {
        elem: Box<Type>,
    },
    Tuple(Vec<Type>),
    Fn {
        params: Vec<Type>,
        ret: Box<Type>,
    },
    Infer,
}
