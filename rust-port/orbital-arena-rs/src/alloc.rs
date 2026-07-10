//! Global-allocator instrumentation used to prove the steady-state frame loop performs
//! zero heap allocation (Article IV / FR-014, research.md R4).
//!
//! This is the crate's one exemption to Article III's `unsafe_code` deny, isolated in this
//! single small, heavily tested module per the constitution's exemption clause.
#![allow(unsafe_code)]

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

static BYTES_ALLOCATED: AtomicUsize = AtomicUsize::new(0);
static BYTES_DEALLOCATED: AtomicUsize = AtomicUsize::new(0);

/// A thin pass-through wrapper around `std::alloc::System` that counts total bytes
/// allocated/deallocated process-wide, so tests can assert a zero delta across a batch of
/// post-warm-up frames.
pub struct CountingAllocator;

// SAFETY: Every method simply forwards `layout`/`ptr` unmodified to `System` and records
// `layout.size()` in an atomic counter before delegating. No memory is read, written, or
// otherwise interpreted directly by this impl; all safety obligations are satisfied by
// `System`'s own `GlobalAlloc` impl receiving an unmodified, valid `Layout`/pointer pair.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        BYTES_ALLOCATED.fetch_add(layout.size(), Ordering::SeqCst);
        // SAFETY: `layout` is forwarded unmodified from this method's own contract.
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        BYTES_DEALLOCATED.fetch_add(layout.size(), Ordering::SeqCst);
        // SAFETY: `ptr`/`layout` are forwarded unmodified from this method's own contract.
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        BYTES_DEALLOCATED.fetch_add(layout.size(), Ordering::SeqCst);
        BYTES_ALLOCATED.fetch_add(new_size, Ordering::SeqCst);
        // SAFETY: `ptr`/`layout`/`new_size` are forwarded unmodified from this method's
        // own contract.
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

/// Snapshot of `(bytes_allocated, bytes_deallocated)` running totals, for before/after
/// comparison in `tests/zero_alloc.rs`.
pub fn snapshot() -> (usize, usize) {
    (
        BYTES_ALLOCATED.load(Ordering::SeqCst),
        BYTES_DEALLOCATED.load(Ordering::SeqCst),
    )
}
