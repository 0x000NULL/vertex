// Per-node-type arenas. Fields are added as the `Item`, `Stmt`, `Expr`, `Ty`,
// and `Pat` enums come online; until then a placeholder keeps the dep
// load-bearing.
pub struct Arena {
    #[allow(dead_code)]
    placeholder: typed_arena::Arena<()>,
}

impl Arena {
    pub fn new() -> Self {
        Self {
            placeholder: typed_arena::Arena::new(),
        }
    }
}

impl Default for Arena {
    fn default() -> Self {
        Self::new()
    }
}
