const SYSCALL_DUP: usize = 24;
const SYSCALL_OPEN: usize = 56;
const SYSCALL_CLOSE: usize = 57;
const SYSCALL_PIPE: usize = 59;
const SYSCALL_READ: usize = 63;
const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_KILL: usize = 129;
const SYSCALL_GET_TIME: usize = 169;
const SYSCALL_GETPID: usize = 172;
const SYSCALL_FORK: usize = 220;
const SYSCALL_EXEC: usize = 221;
const SYSCALL_WAITPID: usize = 260;

const EFAULT: isize = -14;

#[macro_use]
mod macros {
  /// Similar to Rust's try macro or ? operator, but return a user-defined $err
  /// instead of the Err or None variant in $x.
  #[macro_export]
  macro_rules! try_ {
    ($x: expr, $err: expr) => { if let Some(x) = $x { x } else { return $err; } };
  }
}

mod fs;
mod process;
mod uaccess;

use fs::*;
use process::*;
use crate::*;

pub use uaccess::*;

pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
  match syscall_id {
    SYSCALL_DUP => sys_dup(args[0]),
    SYSCALL_OPEN => sys_open(args[0] as _, args[1] as _),
    SYSCALL_CLOSE => sys_close(args[0]),
    SYSCALL_PIPE => sys_pipe(args[0] as _),
    SYSCALL_READ => sys_read(args[0], args[1] as _, args[2]),
    SYSCALL_WRITE => sys_write(args[0], args[1] as _, args[2]),
    SYSCALL_EXIT => sys_exit(args[0] as i32),
    SYSCALL_YIELD => sys_yield(),
    SYSCALL_KILL => sys_kill(args[0], args[1] as _),
    SYSCALL_GET_TIME => *pic::TICKS.get() as _,
    SYSCALL_GETPID => sys_getpid(),
    SYSCALL_FORK => sys_fork(),
    SYSCALL_EXEC => sys_exec(args[0] as _, args[1] as _),
    SYSCALL_WAITPID => sys_waitpid(args[0] as _, args[1] as _),
    _ => {
      println!("[kernel] unknown syscall: {}", syscall_id);
      task::current().exit(-1);
    }
  }
}
