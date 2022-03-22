use crate::{*, trap::SyscallFrame};
use super::*;

pub fn sys_exit(exit_code: i32) -> ! {
  task::current().exit(exit_code)
}

pub fn sys_yield() -> isize {
  task::current_yield();
  0
}

pub fn sys_getpid() -> isize {
  task::current().id as _
}

pub fn sys_fork(f: &SyscallFrame) -> isize {
  task::current().fork(f)
}

pub fn sys_exec(path: *const u8, f: &mut SyscallFrame) -> isize {
  let path = if let Some(x) = read_cstr(path) {x} else { return EFAULT; };
  task::current().exec(&path, f)
}

/// If there is no child process has the same pid as the given, return -1.
/// Else if there is a child process but it is still running, return -2.
pub fn sys_waitpid(pid: isize, exit_code_p: *mut u32) -> isize {
  let (pid, exit_code) = task::current().waitpid(pid);
  if pid >= 0 && !exit_code_p.is_null() {
    exit_code_p.write_user(exit_code as _);
  }
  pid
}
