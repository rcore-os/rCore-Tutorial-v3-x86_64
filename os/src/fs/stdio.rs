use crate::*;
use super::File;

pub struct Stdin;

pub struct Stdout;

impl File for Stdin {
  fn readable(&self) -> bool { true }
  fn writable(&self) -> bool { false }
  fn read(&self, buf: &mut [u8]) -> usize {
    assert_eq!(buf.len(), 1);
    loop {
      if let Some(c) = console::receive() {
        buf[0] = c as _;
        return 1;
      } else {
        task::sched_yield();
      }
    }
  }
  fn write(&self, _: &[u8]) -> usize { panic!("Cannot write to stdin!"); }
}

impl File for Stdout {
  fn readable(&self) -> bool { false }
  fn writable(&self) -> bool { true }
  fn read(&self, _: &mut [u8]) -> usize { panic!("Cannot read from stdout!"); }
  fn write(&self, buf: &[u8]) -> usize {
    if let Ok(str) = core::str::from_utf8(buf) {
      print!("{}", str);
      buf.len()
    } else {
      0
    }
  }
}
