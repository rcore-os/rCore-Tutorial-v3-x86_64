use crate::*;
use super::*;
use core::num::NonZeroUsize;

static FRAME_ALLOCATOR: Cell<FreeListAllocator> = Cell::new(
  FreeListAllocator { current: 0, end: 0, free_list: Vec::new() });

trait FrameAllocator {
  fn alloc(&mut self) -> Option<usize>;
  fn dealloc(&mut self, value: usize);
}

pub struct FreeListAllocator {
  current: usize,
  end: usize,
  free_list: Vec<usize>,
}

impl FreeListAllocator {
  fn alloc(&mut self) -> Option<NonZeroUsize> {
    let mut ret = 0;
    if let Some(x) = self.free_list.pop() {
      ret = x;
    } else if self.current < self.end {
      ret = self.current;
      self.current += PAGE_SIZE;
    };
    NonZeroUsize::new(ret)
  }

  fn dealloc(&mut self, value: usize) {
    assert!(!self.free_list.contains(&value));
    self.free_list.push(value);
  }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct PhysFrame(NonZeroUsize);

impl PhysFrame {
  pub const fn start_pa(&self) -> PhysAddr { PhysAddr(self.0.get()) }

  pub fn alloc() -> Option<Self> {
    FRAME_ALLOCATOR.get().alloc().map(Self)
  }

  pub fn alloc_zero() -> Option<Self> {
    let mut f = Self::alloc()?;
    f.zero();
    Some(f)
  }

  pub fn zero(&mut self) {
    unsafe { core::ptr::write_bytes(self.start_pa().kvaddr().as_ptr(), 0, PAGE_SIZE) }
  }

  pub fn as_slice(&self) -> &mut [u8] {
    unsafe { core::slice::from_raw_parts_mut(self.start_pa().kvaddr().as_ptr(), PAGE_SIZE) }
  }
}

impl Drop for PhysFrame {
  fn drop(&mut self) {
    FRAME_ALLOCATOR.get().dealloc(self.0.get());
  }
}

pub fn init(start: usize, size: usize) {
  FRAME_ALLOCATOR.get().current = start;
  FRAME_ALLOCATOR.get().end = start + size;
}
