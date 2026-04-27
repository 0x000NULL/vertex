use crate::ast::expr::Block;
use crate::ast::ty::Type;
use crate::ast::NodeId;
use crate::span::Span;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Param {
    pub id: NodeId,
    pub span: Span,
    pub name: String,
    pub ty: Type,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FnDef {
    pub id: NodeId,
    pub span: Span,
    pub name: String,
    pub params: Vec<Param>,
    pub ret_ty: Option<Type>,
    pub body: Block,
    pub is_const: bool,
    pub is_unsafe: bool,
    /// `None` = not `extern`. `Some(s)` = `extern` with named ABI string.
    /// An empty string `Some(String::new())` denotes a bare `extern fn` with no ABI literal.
    pub extern_abi: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct StructDef {
    pub id: NodeId,
    pub span: Span,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EnumDef {
    pub id: NodeId,
    pub span: Span,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ImplDef {
    pub id: NodeId,
    pub span: Span,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TraitDef {
    pub id: NodeId,
    pub span: Span,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ModDef {
    pub id: NodeId,
    pub span: Span,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct UseDef {
    pub id: NodeId,
    pub span: Span,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ConstDef {
    pub id: NodeId,
    pub span: Span,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct StaticDef {
    pub id: NodeId,
    pub span: Span,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TypeAliasDef {
    pub id: NodeId,
    pub span: Span,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Item {
    Fn(FnDef),
    Struct(StructDef),
    Enum(EnumDef),
    Impl(ImplDef),
    Trait(TraitDef),
    Mod(ModDef),
    Use(UseDef),
    Const(ConstDef),
    Static(StaticDef),
    TypeAlias(TypeAliasDef),
}

impl Item {
    pub fn id(&self) -> NodeId {
        match self {
            Item::Fn(i) => i.id,
            Item::Struct(i) => i.id,
            Item::Enum(i) => i.id,
            Item::Impl(i) => i.id,
            Item::Trait(i) => i.id,
            Item::Mod(i) => i.id,
            Item::Use(i) => i.id,
            Item::Const(i) => i.id,
            Item::Static(i) => i.id,
            Item::TypeAlias(i) => i.id,
        }
    }

    pub fn span(&self) -> Span {
        match self {
            Item::Fn(i) => i.span,
            Item::Struct(i) => i.span,
            Item::Enum(i) => i.span,
            Item::Impl(i) => i.span,
            Item::Trait(i) => i.span,
            Item::Mod(i) => i.span,
            Item::Use(i) => i.span,
            Item::Const(i) => i.span,
            Item::Static(i) => i.span,
            Item::TypeAlias(i) => i.span,
        }
    }
}
