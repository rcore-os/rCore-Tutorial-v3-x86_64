use crate::{*, mm::*};

core::arch::global_asm!(include_str!("uaccess.S"));

extern "C" {
  pub fn copy_user_start();
  fn copy_user8(dst: *mut u8, src: *const u8) -> usize;
  fn copy_user16(dst: *mut u16, src: *const u16) -> usize;
  fn copy_user32(dst: *mut u32, src: *const u32) -> usize;
  fn copy_user64(dst: *mut usize, src: *const usize) -> usize;
  pub fn copy_user_end();
  pub fn copy_user_fail() -> usize;
}

pub trait ReadUser<T> {
  fn read_user(self) -> Option<T>;
}

pub trait WriteUser<T> {
  fn write_user(self, src: T) -> Option<()>;
}

macro_rules! gen {
  ($t: ty, $f: ident) => {
    impl ReadUser<$t> for *const $t {
      fn read_user(self) -> Option<$t> {
        let mut dst = 0;
        if (self as usize) < PHYS_OFFSET - size_of::<$t>() && unsafe { $f(&mut dst, self) == 0 } {
          Some(dst)
        } else { None }
      }
    }

    impl WriteUser<$t> for *mut $t {
      fn write_user(self, src: $t) -> Option<()> {
        if (self as usize) < PHYS_OFFSET - size_of::<$t>() && unsafe { $f(self, &src) == 0 } {
          Some(())
        } else { None }
      }
    }
  };
}

gen!(u8, copy_user8);
gen!(u16, copy_user16);
gen!(u32, copy_user32);
gen!(usize, copy_user64);

pub fn read_cstr(user: *const u8) -> Option<String> {
  if user.is_null() {
    Some(String::new())
  } else {
    let mut buf = Vec::new();
    for i in 0.. {
      let p = unsafe { user.add(i) };
      let ch = p.read_user()?;
      if ch == 0 { break; }
      buf.push(ch);
    }
    String::from_utf8(buf).ok()
  }
}

pub fn read_cstr_array(user: *const *const u8) -> Option<Vec<String>> {
  if user.is_null() {
    Some(Vec::new())
  } else {
    let mut buf = Vec::new();
    for i in 0.. {
      let p = unsafe { user.add(i) };
      let str = (p as *const usize).read_user()? as *const u8;
      if str.is_null() { break; }
      let str = read_cstr(str)?;
      buf.push(str);
    }
    Some(buf)
  }
}

pub fn validate_buf(root_pa: PhysAddr, ptr: *const u8, len: usize, write: bool) -> Option<&'static mut [u8]> {
  let mut require = PTFlags::PRESENT | PTFlags::USER;
  if write { require |= PTFlags::WRITABLE; }
  let mut p = ptr as _;
  let mut n = len;
  while n != 0 {
    let (_, flags) = query(root_pa, VirtAddr(p))?;
    if !flags.contains(require) { return None; }
    let next = align_down(p) + PAGE_SIZE;
    n -= n.min((next - p) as _);
    p = next;
  }
  return unsafe { Some(core::slice::from_raw_parts_mut(ptr as _, len)) };
}
