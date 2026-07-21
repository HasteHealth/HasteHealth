use haste_reflect::MetaValue;

use crate::{ResolvedValue, allocators::AllocatorTrait};

pub struct Allocator {
    arena: bumpalo::Bump,
}

impl Allocator {
    pub fn new() -> Self {
        Allocator {
            arena: bumpalo::Bump::new(),
        }
    }
}

impl<'a> AllocatorTrait<'a> for Allocator {
    fn allocate_literal<T: MetaValue>(&mut self, value: T) -> &'a dyn MetaValue {
        let literal_ref = self.arena.alloc(value);

        let literal_ptr = std::ptr::from_ref(literal_ref);

        unsafe { &*literal_ptr }
    }

    fn allocate_resolved(&mut self, value: ResolvedValue) -> &'a dyn MetaValue {
        let value = self.arena.alloc(value);

        let ptr = match value {
            ResolvedValue::Box(b) => &raw const **b,
            ResolvedValue::Arc(a) => &raw const **a,
        };

        unsafe { &*ptr }
    }
}
