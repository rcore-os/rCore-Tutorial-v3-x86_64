use crate::*;
use super::*;

#[no_mangle]
pub extern "C" fn syscall_handler(f: &'static mut SyscallFrame) -> isize {
  let r = &f.caller;
  syscall::syscall(r.rax, [r.rdi, r.rsi, r.rdx], f)
}

const PAGE_FAULT: usize = 14;
const TIMER: usize = 32;

#[no_mangle]
pub extern "C" fn trap_handler(f: &'static mut TrapFrame) {
  match f.id {
    PAGE_FAULT if f.rip >= syscall::copy_user_start as usize && f.rip < syscall::copy_user_end as usize => {
      println!("fixup");
      f.rip = syscall::copy_user_fail as usize;
      return;
    }
    TIMER => {
      pic::ack();
      *pic::TICKS.get() += 1;
      task::current_yield();
    }
    _ => {
      println!("trap {:x?}", f);
      task::current().exit(-1);
    }
  }
}
