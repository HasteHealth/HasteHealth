use haste_reflect::MetaValue;

use crate::{ResolvedValue, allocators::AllocatorTrait};

pub struct Allocator {
    _arena: bumpalo::Bump,
}

impl Allocator {
    pub fn new() -> Self {
        Allocator {
            _arena: bumpalo::Bump::new(),
        }
    }
}

impl<'a> AllocatorTrait<'a> for Allocator {
    fn allocate_literal<T: MetaValue>(&mut self, value: T) -> &'a dyn MetaValue {
        let literal_ref = self._arena.alloc(value);

        let literal_ptr = literal_ref as *const _;

        unsafe { &*literal_ptr }
    }

    fn allocate_resolved(&mut self, value: ResolvedValue) -> &'a dyn MetaValue {
        let value = self._arena.alloc(value);

        let ptr = match value {
            ResolvedValue::Box(b) => &**b as *const _,
            ResolvedValue::Arc(a) => &**a,
        };

        unsafe { &*ptr }
    }
}
