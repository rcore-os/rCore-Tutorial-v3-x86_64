use crate::{*, fs::*};
use super::{*, uaccess::*};

pub fn sys_dup(fd: usize) -> isize {
  let t = task::current();
  let file = if let Some(Some(x)) = &t.file_table.get(fd) { x } else { return -1; };
  let file = file.clone();
  t.add_file(file) as _
}

pub fn sys_open(path: *const u8, flags: u32) -> isize {
  let t = task::current();
  let path = try_!(read_cstr(path), EFAULT);
  if let Some(inode) = open_file(&path, OpenFlags::from_bits(flags).unwrap()) {
    t.add_file(inode) as _
  } else {
    -1
  }
}

pub fn sys_close(fd: usize) -> isize {
  let t = task::current();
  if let Some(x) = t.file_table.get_mut(fd) {
    if core::mem::replace(x, None).is_some() { return 0; }
  }
  -1
}

pub fn sys_pipe(pipe: *mut usize) -> isize {
  let t = task::current();
  let (r, w) = make_pipe();
  let (r, w) = (t.add_file(r), t.add_file(w));
  try_!(pipe.write_user(r), EFAULT);
  try_!(unsafe { pipe.add(1) }.write_user(w), EFAULT);
  0
}

pub fn sys_write(fd: usize, ptr: *const u8, len: usize) -> isize {
  let t = task::current();
  let root_pa = t.root_pa();
  let file = if let Some(Some(x)) = &t.file_table.get(fd) { x } else { return -1; };
  if !file.writable() { return -1; }
  let buf = try_!(validate_buf(root_pa, ptr, len, false), EFAULT);
  file.write(buf) as _
}

pub fn sys_read(fd: usize, ptr: *mut u8, len: usize) -> isize {
  let t = task::current();
  let root_pa = t.root_pa();
  let file = if let Some(Some(x)) = &t.file_table.get(fd) { x } else { return -1; };
  if !file.readable() { return -1; }
  let buf = try_!(validate_buf(root_pa, ptr, len, true), EFAULT);
  file.read(buf) as _
}
