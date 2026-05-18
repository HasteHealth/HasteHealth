use std::marker::PhantomData;

use haste_reflect::MetaValue;

use crate::{ResolvedValue, allocators::AllocatorTrait};

/// Need a means to store objects that are created during evaluation.
#[allow(dead_code)]
pub struct Allocator<'a> {
    pub context: Vec<ResolvedValue>,
    _marker: PhantomData<&'a dyn MetaValue>,
}

impl<'a> Allocator<'a> {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Allocator {
            context: Vec::new(),
            _marker: PhantomData,
        }
    }
}

impl<'a> AllocatorTrait<'a> for Allocator<'a> {
    fn allocate_literal<T: MetaValue>(&mut self, value: T) -> &'a dyn MetaValue {
        let literal = ResolvedValue::Box(Box::new(value));
        self.allocate_resolved(literal)
    }

    fn allocate_resolved(&mut self, value: ResolvedValue) -> &'a dyn MetaValue {
        self.context.push(value);

        // Should be safe to unwrap as value guaranteed to be non-empty from above call.
        let ptr = match &self.context.last().unwrap() {
            ResolvedValue::Box(b) => &**b as *const _,
            ResolvedValue::Arc(a) => &**a,
        };

        unsafe { &*ptr }
    }
}
