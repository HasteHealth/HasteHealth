use haste_reflect::MetaValue;

use crate::{ResolvedValue, allocators::AllocatorTrait};

pub struct Allocator {
    arena: bumpalo::Bump,
    now: Option<chrono::DateTime<chrono::Utc>>,
    repeat_items: usize,
    string_bytes: usize,
}

impl Allocator {
    pub fn new() -> Self {
        Allocator {
            arena: bumpalo::Bump::new(),
            now: None,
            repeat_items: 0,
            string_bytes: 0,
        }
    }

    /// The instant of the first call, so `now()`, `today()` and `timeOfDay()`
    /// agree throughout one evaluation.
    pub fn now(&mut self) -> chrono::DateTime<chrono::Utc> {
        *self.now.get_or_insert_with(chrono::Utc::now)
    }

    /// Adds `count` to the items `repeat()` has produced in this evaluation, nested
    /// calls included, and returns the new total.
    pub fn add_repeat_items(&mut self, count: usize) -> usize {
        self.repeat_items = self.repeat_items.saturating_add(count);
        self.repeat_items
    }

    /// The memory the strings this evaluation created take, in bytes.
    pub fn string_bytes(&self) -> usize {
        self.string_bytes
    }

    /// Adds `bytes` to the memory the strings this evaluation created take and returns
    /// the new total.
    pub fn add_string_bytes(&mut self, bytes: usize) -> usize {
        self.string_bytes = self.string_bytes.saturating_add(bytes);
        self.string_bytes
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
