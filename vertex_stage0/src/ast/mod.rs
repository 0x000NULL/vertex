pub mod arena;

pub use arena::Arena;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct NodeId(pub u32);
