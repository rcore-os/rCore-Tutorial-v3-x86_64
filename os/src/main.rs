#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(default_alloc_error_handler)]
#![feature(const_maybe_uninit_zeroed)]
#![feature(new_uninit)]
#![feature(const_btree_new)]

extern crate alloc;

use core::{mem, cell::UnsafeCell, ops::{Deref, DerefMut}, panic::PanicInfo};

pub use mem::{transmute, size_of, size_of_val};
pub use alloc::{vec, vec::Vec, boxed::Box, string::String};
pub use easy_fs::BlockDevice;

#[macro_use]
mod console;

mod drivers;
mod fs;
mod mm;
mod syscall;
mod task;
mod trap;
mod pic;
mod x86_64;

/// The entry point of kernel
#[no_mangle]
extern "C" fn _start(boot_info: &'static rboot::BootInfo) -> ! {
  console::init();
  trap::init();
  pic::init();

  let (mut start, mut size) = (0, 0);
  for &region in &boot_info.memory_map {
    if region.ty == rboot::MemoryType::CONVENTIONAL && region.page_count > size {
      size = region.page_count;
      start = region.phys_start;
    }
  }
  size *= mm::PAGE_SIZE as u64;
  println!("[kernel] physical frames start = {:x}, size = {:x}", start, size);
  mm::init(start as _, size as _);

  drivers::init();
  fs::init();
  task::init();
}

#[inline(always)]
pub const fn zero<T>() -> T {
  unsafe { mem::MaybeUninit::zeroed().assume_init() }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct Cell<T>(UnsafeCell<T>);

unsafe impl<T> Sync for Cell<T> {}

impl<T> Cell<T> {
  /// User is responsible to guarantee that inner struct is only used in
  /// uniprocessor.
  #[inline(always)]
  pub const fn new(val: T) -> Self {
    Self(UnsafeCell::new(val))
  }

  #[inline(always)]
  pub fn get(&self) -> &mut T {
    unsafe { &mut *self.0.get() }
  }
}

impl<T> Deref for Cell<T> {
  type Target = T;
  #[inline(always)]
  fn deref(&self) -> &Self::Target { self.get() }
}

impl<T> DerefMut for Cell<T> {
  #[inline(always)]
  fn deref_mut(&mut self) -> &mut Self::Target { self.get() }
}

#[no_mangle]
fn rust_oom() -> ! { panic!("rust_oom"); }

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
  if let Some(l) = info.location() {
    println!("[kernel] Panicked at {}:{} {}", l.file(), l.line(), info.message().unwrap());
  } else {
    println!("[kernel] Panicked: {}", info.message().unwrap());
  }
  loop {}
}

