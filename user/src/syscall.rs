use core::arch::asm;

const SYSCALL_OPEN: usize = 56;
const SYSCALL_CLOSE: usize = 57;
const SYSCALL_READ: usize = 63;
const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_GET_TIME: usize = 169;
const SYSCALL_GETPID: usize = 172;
const SYSCALL_FORK: usize = 220;
const SYSCALL_EXEC: usize = 221;
const SYSCALL_WAITPID: usize = 260;

#[inline(always)]
fn syscall(id: usize, arg0: usize, arg1: usize, arg2: usize) -> isize {
  let ret;
  unsafe {
    asm!(
      "syscall",
      in("rax") id, in("rdi") arg0, in("rsi") arg1, in("rdx") arg2,
      out("rcx") _, out("r11") _, // clobbered by syscall
      lateout("rax") ret
    );
  }
  ret
}

pub fn sys_open(path: &str, flags: u32) -> isize {
    syscall(SYSCALL_OPEN, path.as_ptr() as _, flags as _, 0)
}

pub fn sys_close(fd: usize) -> isize {
  syscall(SYSCALL_CLOSE, fd, 0, 0)
}

pub fn sys_read(fd: usize, buf: &mut [u8]) -> isize {
  syscall(SYSCALL_READ, fd, buf.as_mut_ptr() as _, buf.len())
}

pub fn sys_write(fd: usize, buf: &[u8]) -> isize {
  syscall(SYSCALL_WRITE, fd, buf.as_ptr() as _, buf.len())
}

pub fn sys_exit(exit_code: i32) -> ! {
  syscall(SYSCALL_EXIT, exit_code as _, 0, 0);
  panic!("sys_exit never returns!");
}

pub fn sys_yield() -> isize {
  syscall(SYSCALL_YIELD, 0, 0, 0)
}

pub fn sys_get_time() -> isize {
  syscall(SYSCALL_GET_TIME, 0, 0, 0)
}

pub fn sys_getpid() -> isize {
  syscall(SYSCALL_GETPID, 0, 0, 0)
}

pub fn sys_fork() -> isize {
  syscall(SYSCALL_FORK, 0, 0, 0)
}

pub fn sys_exec(path: &str) -> isize {
  syscall(SYSCALL_EXEC, path.as_ptr() as _, 0, 0)
}

pub fn sys_waitpid(pid: isize, exit_code: *mut i32) -> isize {
  syscall(SYSCALL_WAITPID, pid as _, exit_code as _, 0)
}
