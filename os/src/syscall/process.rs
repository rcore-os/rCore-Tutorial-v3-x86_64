use crate::{*, task::*, mm::*};
use super::*;

pub fn sys_exit(exit_code: i32) -> ! {
  task::current().exit(exit_code)
}

pub fn sys_yield() -> isize {
  task::sched_yield();
  0
}

pub fn sys_kill(pid: usize, signal: u32) -> isize {
  if let Some(p) = PID2PROC.get().get_mut(&pid) {
    if let Some(signal) = SignalFlags::from_bits(signal as _) {
      p.add_signal(signal);
      return 0;
    }
  }
  -1
}

pub fn sys_getpid() -> isize {
  task::current().proc.pid as _
}

pub fn sys_fork() -> isize {
  task::current().proc.fork().pid as _
}

pub fn sys_exec(path: *const u8, args: *const *const u8) -> isize {
  let path = try_!(read_cstr(path), EFAULT);
  let args = try_!(read_cstr_array(args), EFAULT);
  task::current().proc.exec(&path, args)
}

/// If there is no child process has the same pid as the given, return -1.
/// Else if there is a child process but it is still running, return -2.
pub fn sys_waitpid(pid: isize, exit_code_p: *mut u32) -> isize {
  let (pid, exit_code) = task::current().proc.waitpid(pid);
  if pid >= 0 && !exit_code_p.is_null() {
    try_!(exit_code_p.write_user(exit_code as _), EFAULT);
  }
  pid
}

pub fn sys_thread_create(entry: usize, arg: usize) -> isize {
  let t = current();
  let (t1, need_stack) = Task::new(t.proc, user_task_entry, 0);
  let stack = USTACK_TOP - t1.tid * USTACK_SIZE;
  if need_stack {
    t.proc.vm.as_mut().unwrap().insert(MapArea::new(VirtAddr(stack - USTACK_SIZE), USTACK_SIZE,
      PTFlags::PRESENT | PTFlags::WRITABLE | PTFlags::USER));
  }
  let f = t1.syscall_frame();
  f.caller.rcx = entry;
  f.caller.r11 = x86_64::RFLAGS_IF;
  f.callee.rsp = stack;
  f.caller.rdi = arg;
  t1.tid as _
}

pub fn sys_gettid() -> isize {
  task::current().tid as _
}

pub fn sys_waittid(tid: usize) -> isize {
  let t = current();
  // A thread cannot wait for itself.
  if t.tid == tid { return -1; }
  let t1 = try_!(t.proc.tasks.get_mut(tid), -1);
  if t1.status == TaskStatus::Zombie {
    t1.status = TaskStatus::Waited;
    t1.exit_code as _
  } else {
    -2 // waited thread has not exited
  }
}
