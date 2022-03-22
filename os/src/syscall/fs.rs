use crate::*;
use super::*;

const FD_STDIN: usize = 0;
const FD_STDOUT: usize = 1;
const CHUNK_SIZE: usize = 256;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
  match fd {
    FD_STDOUT => {
      let mut n = 0;
      while n < len {
        let chunk_len = CHUNK_SIZE.min(len - n);
        let chunk = read_array::<CHUNK_SIZE>(unsafe { buf.add(n) }, chunk_len);
        if let Some(str) = chunk.as_ref().and_then(|x| core::str::from_utf8(&x[..chunk_len]).ok()) {
          print!("{}", str);
          n += chunk_len;
        } else {
          return EFAULT;
        }
      }
      n as isize
    }
    _ => {
      panic!("Unsupported fd in sys_write!");
    }
  }
}

pub fn sys_read(fd: usize, buf: *mut u8, len: usize) -> isize {
  match fd {
    FD_STDIN => {
      assert_eq!(len, 1, "Only support len = 1 in sys_read!");
      loop {
        if let Some(c) = console::receive() {
          return if buf.write_user(c).is_some() { 1 } else { EFAULT };
        } else {
          task::current_yield();
        }
      }
    }
    _ => {
      panic!("Unsupported fd in sys_read!");
    }
  }
}
