use core::arch::{asm, global_asm};
use core::ops::Deref;
use core::sync::atomic::{AtomicUsize, Ordering};

use crate::config::{ KERNEL_STACK_SIZE, USER_STACK_SIZE};
use crate::loader;
use crate::loader::get_app_name;
use crate::trap::{CallerRegs, set_user_rsp, syscall_return};

const APP_BASE_ADDRESS: usize = 0xffffff0001000000;
const APP_SIZE_LIMIT: usize = 0x20000;

#[no_mangle]
#[link_section = "._user_section"]
pub static mut APP_DST: [u8; APP_SIZE_LIMIT] = [0; APP_SIZE_LIMIT];

#[repr(align(4096))]
pub struct PageAligned<T>(T);

impl<T> Deref for PageAligned<T> {
  type Target = T;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

static mut KERNEL_STACK: PageAligned<[u8; KERNEL_STACK_SIZE]> = PageAligned([0; KERNEL_STACK_SIZE]);
const USER_STACK: usize = APP_BASE_ADDRESS + APP_SIZE_LIMIT;

struct AppManager {
  current_app: AtomicUsize,
}

impl AppManager {
  pub const fn new() -> Self {
    Self { current_app: AtomicUsize::new(0) }
  }

  pub unsafe fn load_next_app(&self) {
    let app_id = self.current_app.fetch_add(1, Ordering::SeqCst);
    if app_id >= loader::get_app_count() {
      panic!("All applications completed!");
    }
    println!("[kernel] Loading app_{} {}", app_id, get_app_name(app_id));
    let app_data = loader::get_app_data(app_id);
    assert_eq!(APP_DST.as_ptr() as usize, APP_BASE_ADDRESS);
    assert!(app_data.len() < APP_SIZE_LIMIT);
    let app_dst = core::slice::from_raw_parts_mut(APP_BASE_ADDRESS as _, app_data.len());
    app_dst.copy_from_slice(app_data);
  }
}

static APP_MANAGER: AppManager = AppManager::new();

pub fn init() {
  loader::list_apps();
}

pub fn run_next_app() -> ! {
  unsafe {
    APP_MANAGER.load_next_app();
    let ctx = &mut *((KERNEL_STACK.as_ptr_range().end as usize - core::mem::size_of::<CallerRegs>()) as *mut CallerRegs);
    ctx.rcx = APP_BASE_ADDRESS as _;
    ctx.r11 = 0;
    set_user_rsp(USER_STACK as _);
    syscall_return(ctx);
  };
}
