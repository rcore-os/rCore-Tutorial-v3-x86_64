use crate::*;
use buddy_system_allocator::Heap;
use core::{alloc::{GlobalAlloc, Layout}, ptr::NonNull};

const KERNEL_HEAP_SIZE: usize = 0x80_0000;

struct LockedHeap(Cell<Heap<32>>);

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap(Cell::new(Heap::new()));

unsafe impl GlobalAlloc for LockedHeap {
  unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
    self.0.get().alloc(layout).ok().map_or(0 as _, |p| p.as_ptr())
  }

  unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
    self.0.get().dealloc(NonNull::new_unchecked(ptr), layout)
  }
}

static mut HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

pub fn init() {
  unsafe { HEAP_ALLOCATOR.0.get().init(HEAP_SPACE.as_ptr() as _, KERNEL_HEAP_SIZE); }
}
