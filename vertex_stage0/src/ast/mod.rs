pub mod arena;
pub mod item;

pub use arena::Arena;
pub use item::Item;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct NodeId(pub u32);
