use core::arch::asm;

const SYSCALL_DUP: usize = 24;
const SYSCALL_OPEN: usize = 56;
const SYSCALL_CLOSE: usize = 57;
const SYSCALL_PIPE: usize = 59;
const SYSCALL_READ: usize = 63;
const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_SLEEP: usize = 101;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_KILL: usize = 129;
const SYSCALL_GET_TIME: usize = 169;
const SYSCALL_GETPID: usize = 172;
const SYSCALL_FORK: usize = 220;
const SYSCALL_EXEC: usize = 221;
const SYSCALL_WAITPID: usize = 260;
const SYSCALL_THREAD_CREATE: usize = 1000;
const SYSCALL_GETTID: usize = 1001;
const SYSCALL_WAITTID: usize = 1002;
const SYSCALL_MUTEX_CREATE: usize = 1010;
const SYSCALL_MUTEX_LOCK: usize = 1011;
const SYSCALL_MUTEX_UNLOCK: usize = 1012;
const SYSCALL_SEMAPHORE_CREATE: usize = 1020;
const SYSCALL_SEMAPHORE_UP: usize = 1021;
const SYSCALL_SEMAPHORE_DOWN: usize = 1022;
const SYSCALL_CONDVAR_CREATE: usize = 1030;
const SYSCALL_CONDVAR_SIGNAL: usize = 1031;
const SYSCALL_CONDVAR_WAIT: usize = 1032;

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

pub fn sys_dup(fd: usize) -> isize {
  syscall(SYSCALL_DUP, fd, 0, 0)
}

pub fn sys_open(path: &str, flags: u32) -> isize {
  syscall(SYSCALL_OPEN, path.as_ptr() as _, flags as _, 0)
}

pub fn sys_close(fd: usize) -> isize {
  syscall(SYSCALL_CLOSE, fd, 0, 0)
}

pub fn sys_pipe(pipe: &mut [usize]) -> isize {
  syscall(SYSCALL_PIPE, pipe.as_ptr() as _, 0, 0)
}

pub fn sys_read(fd: usize, buf: &mut [u8]) -> isize {
  syscall(SYSCALL_READ, fd, buf.as_ptr() as _, buf.len())
}

pub fn sys_write(fd: usize, buf: &[u8]) -> isize {
  syscall(SYSCALL_WRITE, fd, buf.as_ptr() as _, buf.len())
}

pub fn sys_exit(exit_code: i32) -> ! {
  syscall(SYSCALL_EXIT, exit_code as _, 0, 0);
  panic!("sys_exit never returns!");
}

pub fn sys_sleep(sleep_ms: usize) -> isize {
  syscall(SYSCALL_SLEEP, sleep_ms, 0, 0)
}

pub fn sys_yield() -> isize {
  syscall(SYSCALL_YIELD, 0, 0, 0)
}

pub fn sys_kill(pid: usize, signal: i32) -> isize {
  syscall(SYSCALL_KILL, pid, signal as _, 0)
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

pub fn sys_exec(path: &str, args: &[*const u8]) -> isize {
  syscall(SYSCALL_EXEC, path.as_ptr() as _, args.as_ptr() as _, 0)
}

pub fn sys_waitpid(pid: isize, exit_code: *mut i32) -> isize {
  syscall(SYSCALL_WAITPID, pid as _, exit_code as _, 0)
}

pub fn sys_thread_create(entry: usize, arg: usize) -> isize {
  syscall(SYSCALL_THREAD_CREATE, entry, arg, 0)
}

pub fn sys_gettid() -> isize {
  syscall(SYSCALL_GETTID, 0, 0, 0)
}

pub fn sys_waittid(tid: usize) -> isize {
  syscall(SYSCALL_WAITTID, tid, 0, 0)
}

pub fn sys_mutex_create(blocking: bool) -> isize {
  syscall(SYSCALL_MUTEX_CREATE, blocking as _, 0, 0)
}

pub fn sys_mutex_lock(id: usize) -> isize {
  syscall(SYSCALL_MUTEX_LOCK, id, 0, 0)
}

pub fn sys_mutex_unlock(id: usize) -> isize {
  syscall(SYSCALL_MUTEX_UNLOCK, id, 0, 0)
}

pub fn sys_semaphore_create(res_count: usize) -> isize {
  syscall(SYSCALL_SEMAPHORE_CREATE, res_count, 0, 0)
}

pub fn sys_semaphore_up(sem_id: usize) -> isize {
  syscall(SYSCALL_SEMAPHORE_UP, sem_id, 0, 0)
}

pub fn sys_semaphore_down(sem_id: usize) -> isize {
  syscall(SYSCALL_SEMAPHORE_DOWN, sem_id, 0, 0)
}

pub fn sys_condvar_create(_arg: usize) -> isize {
  syscall(SYSCALL_CONDVAR_CREATE, _arg, 0, 0)
}

pub fn sys_condvar_signal(condvar_id: usize) -> isize {
  syscall(SYSCALL_CONDVAR_SIGNAL, condvar_id, 0, 0)
}

pub fn sys_condvar_wait(condvar_id: usize, mutex_id: usize) -> isize {
  syscall(SYSCALL_CONDVAR_WAIT, condvar_id, mutex_id, 0)
}
