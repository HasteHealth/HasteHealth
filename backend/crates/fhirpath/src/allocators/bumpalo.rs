use std::ptr::NonNull;

use haste_reflect::MetaValue;

use crate::{ResolvedValue, allocators::AllocatorTrait};

/// A value in the arena that owns memory outside it, and the one allocated before it.
struct Owner {
    value: *mut dyn MetaValue,
    previous: Option<NonNull<Owner>>,
}

pub struct Allocator {
    arena: bumpalo::Bump,
    // The arena's values that own memory outside it, latest first and listed in the
    // arena itself. An arena frees its own memory without dropping what it holds, so
    // they are dropped from here.
    owners: Option<NonNull<Owner>>,
    // Resolved values own all their memory, so they are kept as they are.
    resolved: Vec<ResolvedValue>,
    now: Option<chrono::DateTime<chrono::Utc>>,
    repeat_items: usize,
}

// SAFETY: `owners` points into `arena`, which moves with the allocator, and a
// `MetaValue` is `Send`.
unsafe impl Send for Allocator {}

impl Allocator {
    pub fn new() -> Self {
        Allocator {
            arena: bumpalo::Bump::new(),
            owners: None,
            resolved: Vec::new(),
            now: None,
            repeat_items: 0,
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
}

impl Drop for Allocator {
    fn drop(&mut self) {
        let mut owners = self.owners.take();
        while let Some(owner) = owners {
            // SAFETY: an owner and its value are in `arena`, which is still alive, and
            // the value is dropped only here.
            unsafe {
                let owner = owner.as_ref();
                std::ptr::drop_in_place(owner.value);
                owners = owner.previous;
            }
        }
    }
}

impl<'a> AllocatorTrait<'a> for Allocator {
    fn allocate_literal<T: MetaValue>(&mut self, value: T) -> &'a dyn MetaValue {
        let literal_ref: &mut dyn MetaValue = self.arena.alloc(value);

        let literal_ptr = std::ptr::from_mut(literal_ref);
        if std::mem::needs_drop::<T>() {
            let owner = self.arena.alloc(Owner {
                value: literal_ptr,
                previous: self.owners,
            });
            self.owners = Some(NonNull::from(owner));
        }

        unsafe { &*literal_ptr }
    }

    fn allocate_resolved(&mut self, value: ResolvedValue) -> &'a dyn MetaValue {
        let ptr = match &value {
            ResolvedValue::Box(b) => &raw const **b,
            ResolvedValue::Arc(a) => &raw const **a,
        };
        self.resolved.push(value);

        unsafe { &*ptr }
    }
}
