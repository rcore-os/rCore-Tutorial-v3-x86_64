use crate::*;
use super::*;

#[no_mangle]
pub extern "C" fn syscall_handler(f: &'static mut SyscallFrame) -> isize {
  let r = &f.regs;
  syscall::syscall(r.rax, [r.rdi, r.rsi, r.rdx])
}

#[no_mangle]
pub extern "C" fn trap_handler(f: &'static mut TrapFrame) {
  match f.id {
    32 => {
      pic::ack();
      *pic::TICKS.get() += 1;
      task::current_yield();
    }
    _ => {
      println!("trap {:x?}", f);
      task::current_exit(-1);
    }
  }
}
