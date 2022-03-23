mod frame_allocator;
mod heap_allocator;
mod memory_set;
mod page_table;

use core::fmt;

pub use frame_allocator::*;
pub use memory_set::*;
pub use page_table::*;

/// Total mapped memory in kernel. Region [ekernel, KERNEL_SIZE) is allocated as physical frames.
/// It occupies less than one top-level entry in the kernel page table,
/// starting from p4_index 0xff00_0000_0000 / ENTRY_COUNT^2 / PAGE_SIZE (0x1fe).
pub const KERNEL_OFFSET: usize = 0xffffff00_00000000;
pub const PHYS_OFFSET: usize = 0xffff8000_00000000;

pub const PAGE_SIZE: usize = 4096;
pub const ENTRY_COUNT: usize = 512;

bitflags::bitflags! {
  /// Possible flags for a page table entry.
  pub struct PTFlags: usize {
    /// Specifies whether the mapped frame or page table is loaded in memory.
    const PRESENT =         1;
    /// Controls whether writes to the mapped frames are allowed.
    const WRITABLE =        1 << 1;
    /// Controls whether accesses from userspace (i.e. ring 3) are permitted.
    const USER = 1 << 2;
    /// If this bit is set, a “write-through” policy is used for the cache, else a “write-back”
    /// policy is used.
    const WRITE_THROUGH =   1 << 3;
    /// Disables caching for the pointed entry is cacheable.
    const NO_CACHE =        1 << 4;
    /// Indicates that the mapping is present in all address spaces, so it isn't flushed from
    /// the TLB on an address space switch.
    const GLOBAL =          1 << 8;
  }
}

pub fn init(start: usize, size: usize) {
  heap_allocator::init();
  frame_allocator::init(start, size);
  page_table::init();
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
#[repr(transparent)]
pub struct PhysAddr(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
#[repr(transparent)]
pub struct VirtAddr(pub usize);

pub const fn phys_to_virt(pa: usize) -> usize { pa + PHYS_OFFSET }

pub const fn virt_to_phys(va: usize) -> usize { va - PHYS_OFFSET }

impl fmt::Debug for PhysAddr {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "PA:{:#x}", self.0)
  }
}

impl fmt::Debug for VirtAddr {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "VA:{:#x}", self.0)
  }
}

impl PhysAddr {
  pub const fn kvaddr(self) -> VirtAddr { VirtAddr(phys_to_virt(self.0)) }
  pub const fn align_down(self) -> Self { Self(align_down(self.0)) }
  pub const fn align_up(self) -> Self { Self(align_up(self.0)) }
  pub const fn page_offset(self) -> usize { page_offset(self.0) }
  pub const fn is_aligned(self) -> bool { is_aligned(self.0) }
}

impl VirtAddr {
  pub const fn as_ptr(self) -> *mut u8 { self.0 as _ }
  pub const fn align_down(self) -> Self { Self(align_down(self.0)) }
  pub const fn align_up(self) -> Self { Self(align_up(self.0)) }
  pub const fn page_offset(self) -> usize { page_offset(self.0) }
  pub const fn is_aligned(self) -> bool { is_aligned(self.0) }
}

pub const fn align_down(p: usize) -> usize { p & !(PAGE_SIZE - 1) }

pub const fn align_up(p: usize) -> usize { (p + PAGE_SIZE - 1) & !(PAGE_SIZE - 1) }

pub const fn page_offset(p: usize) -> usize { p & (PAGE_SIZE - 1) }

pub const fn is_aligned(p: usize) -> bool { page_offset(p) == 0 }
