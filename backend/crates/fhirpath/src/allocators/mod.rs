use haste_reflect::MetaValue;

use crate::ResolvedValue;

pub mod bumpalo;
pub mod vec_allocator;

pub trait AllocatorTrait<'a> {
    fn allocate_literal<T: MetaValue>(&mut self, value: T) -> &'a dyn MetaValue;
    fn allocate_resolved(&mut self, value: ResolvedValue) -> &'a dyn MetaValue;
}
