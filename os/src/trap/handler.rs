use crate::{*, task::*};
use super::*;

#[no_mangle]
pub extern "C" fn syscall_handler(f: &'static mut SyscallFrame) -> isize {
  let r = &f.caller;
  let ret = syscall::syscall(r.rax, [r.rdi, r.rsi, r.rdx]);
  current_check_signal();
  ret
}

const DIVIDE_BY_ZERO: usize = 0;
const INVALID_OPCODE: usize = 6;
const SEGMENT_NOT_PRESENT: usize = 11;
const STACK_SEGMENT_FAULT: usize = 12;
const GENERAL_PROTECTION_FAULT: usize = 13;
const PAGE_FAULT: usize = 14;
const TIMER: usize = 32;

#[no_mangle]
pub extern "C" fn trap_handler(f: &'static mut TrapFrame) {
  match f.id {
    DIVIDE_BY_ZERO =>  current().add_signal(SignalFlags::SIGFPE),
    INVALID_OPCODE => current().add_signal(SignalFlags::SIGILL),
    SEGMENT_NOT_PRESENT | STACK_SEGMENT_FAULT | GENERAL_PROTECTION_FAULT =>
      current().add_signal(SignalFlags::SIGSEGV),
    PAGE_FAULT => if f.rip >= syscall::copy_user_start as usize && f.rip < syscall::copy_user_end as usize {
      println!("[kernel] copy_user_fail");
      f.rip = syscall::copy_user_fail as usize;
      return;
    } else {
      current().add_signal(SignalFlags::SIGSEGV);
    }
    TIMER => {
      pic::ack();
      *pic::TICKS.get() += 1;
      sched_yield();
    }
    _ => {
      println!("[kernel] unknown trap {:x?}", f);
      current().exit(-1);
    }
  }
  current_check_signal();
}
