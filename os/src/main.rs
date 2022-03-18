#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(default_alloc_error_handler)]
#![feature(const_maybe_uninit_zeroed)]

#[macro_use]
extern crate bitflags;

mod x86_64;

#[macro_use]
mod console;
mod batch;
// mod arch;
mod config;
// mod entry;
// mod gicv2;
mod lang_items;
mod loader;
// mod mm;
// mod psci;
// mod sync;
mod syscall;
// mod task;
// mod timer;
mod trap;
// mod utils;

use rboot::BootInfo;


/// The entry point of kernel
#[no_mangle] // don't mangle the name of this function
pub extern "C" fn _start(boot_info: &'static BootInfo) -> ! {
  console::init();
  trap::init();

  // use crate::x86_64::*;
  // let gdp = sgdt();
  // let gdt = unsafe { core::slice::from_raw_parts(gdp.base as *mut u64, 7) };
  // println!("{:?} {:x?}", gdp, gdt);
  // cf92000000ffff, cf9f000000ffff, cf93000000ffff, cf9a000000ffff, 0, cf93000000ffff, af9a000000ffff

  println!("[kernel] Hello, world!");

  // let x = !0u64;
  // let y = unsafe {*(x as *const u8)};

  batch::init();
  batch::run_next_app();

  // mm::init();
  // mm::remap_test();
  // trap::init();
  // trap::enable_timer_interrupt();
  // timer::set_next_trigger();
  // fs::list_apps();
  // task::add_initproc();
  // task::run_tasks();
  panic!("Unreachable in rust_main!");
}
