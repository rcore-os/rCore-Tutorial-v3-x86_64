use crate::*;

pub fn sys_exit(exit_code: i32) -> ! {
  task::current_exit(exit_code)
}

pub fn sys_yield() -> isize {
  task::current_yield();
  0
}
