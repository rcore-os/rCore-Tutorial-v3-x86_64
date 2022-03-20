const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_GET_TIME: usize = 169;

mod fs;
mod process;

use crate::*;
use self::{fs::*, process::*};

pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
  let ret = match syscall_id {
    SYSCALL_WRITE => sys_write(args[0], args[1] as _, args[2]),
    SYSCALL_EXIT => sys_exit(args[0] as _),
    SYSCALL_YIELD => sys_yield(),
    SYSCALL_GET_TIME => *pic::TICKS.get() as _,
    _ => {
      println!("Unsupported syscall: {}", syscall_id);
      task::current_exit(-1);
    }
  };
  ret
}
