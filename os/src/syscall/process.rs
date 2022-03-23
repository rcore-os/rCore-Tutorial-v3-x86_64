use crate::{*, task::*};
use super::*;

pub fn sys_exit(exit_code: i32) -> ! {
  task::current().exit(exit_code)
}

pub fn sys_yield() -> isize {
  task::sched_yield();
  0
}

pub fn sys_kill(pid: usize, signal: u32) -> isize {
  if let Some(t) = PID2TASK.get().get_mut(&pid) {
    if let Some(signal) = SignalFlags::from_bits(signal as _) {
      t.add_signal(signal);
      return 0;
    }
  }
  -1
}

pub fn sys_getpid() -> isize {
  task::current().id as _
}

pub fn sys_fork() -> isize {
  task::current().fork()
}

pub fn sys_exec(path: *const u8, args: *const *const u8) -> isize {
  let path = try_!(read_cstr(path), EFAULT);
  let args = try_!(read_cstr_array(args), EFAULT);
  task::current().exec(&path, args)
}

/// If there is no child process has the same pid as the given, return -1.
/// Else if there is a child process but it is still running, return -2.
pub fn sys_waitpid(pid: isize, exit_code_p: *mut u32) -> isize {
  let (pid, exit_code) = task::current().waitpid(pid);
  if pid >= 0 && !exit_code_p.is_null() {
    try_!(exit_code_p.write_user(exit_code as _), EFAULT);
  }
  pid
}
