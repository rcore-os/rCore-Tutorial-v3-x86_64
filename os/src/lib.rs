#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(default_alloc_error_handler)]
#![feature(const_maybe_uninit_zeroed)]

use core::{mem, cell::UnsafeCell, ops::{Deref, DerefMut}, panic::PanicInfo};

#[macro_use]
mod console;

mod syscall;
mod task;
mod trap;
mod loader;
mod pic;
mod x86_64;

/// The entry point of kernel
#[no_mangle]
pub extern "C" fn _start(boot_info: &'static rboot::BootInfo) -> ! {
  console::init();
  trap::init();

  println!("[kernel] Hello, world!");

  pic::init();

  loader::list_apps();
  task::init();

  // mm::init();
  // mm::remap_test();

  // fs::list_apps();
  // task::add_initproc();
  // task::run_tasks();
  panic!("Unreachable in rust_main!");
}


pub use mem::{transmute, size_of, size_of_val};

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
fn rust_oom() -> ! {
  panic!("rust_oom");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
  if let Some(location) = info.location() {
    println!(
      "[kernel] Panicked at {}:{} {}",
      location.file(),
      location.line(),
      info.message().unwrap()
    );
  } else {
    println!("[kernel] Panicked: {}", info.message().unwrap());
  }
  loop {}
}
