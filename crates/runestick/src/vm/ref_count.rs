use crate::value::{Slot, Value};
use std::cell::Cell;
use std::collections::VecDeque;

#[derive(Clone, Copy)]
struct RawGuard<'a> {
    ref_count: &'a RefCount,
    reap_queue: *const VecDeque<Slot>,
}

impl RawGuard<'_> {
    /// Increase the reference count.
    fn inc(&self) {
        self.ref_count.inc();
    }

    /// Decrease the reference count.
    fn dec(&self, value: Value) {
        if self.ref_count.dec() {
            let slot = self
                .into_slot()
                .expect("only slotted values should be refcounted");

            unsafe { (*(self.reap_queue as *mut VecDeque<Slot>)).push_back(slot) }
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct RefCount {
    count: Cell<usize>,
}

impl RefCount {
    /// Construct a new refcount.
    pub(super) const fn new(count: usize) -> Self {
        Self {
            count: Cell::new(count),
        }
    }

    /// Increment the refcount.
    pub(super) fn inc(&self) {
        let count = self.count.get().checked_add(1).expect("overflow");
        self.count.set(count);
    }

    /// Decrement the refcount.
    pub(super) fn dec(&self) -> bool {
        let count = self.count.get().checked_sub(1).expect("underflow");
        self.count.set(count);
        count == 0
    }
}
