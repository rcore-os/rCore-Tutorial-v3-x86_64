use crate::*;
use super::File;
use alloc::rc::Weak;

pub struct Pipe {
  writable: bool,
  buf: Rc<Cell<PipeBuffer>>,
}

struct PipeBuffer {
  buf: VecDeque<u8>,
  write_end: Weak<Pipe>,
}

/// Return (read_end, write_end)
pub fn make_pipe() -> (Rc<Pipe>, Rc<Pipe>) {
  let buf = Rc::new(Cell::new(PipeBuffer { buf: VecDeque::new(), write_end: Weak::new() }));
  let w = Rc::new(Pipe { writable: true, buf: buf.clone() });
  buf.get().write_end = Rc::downgrade(&w);
  let r = Rc::new(Pipe { writable: false, buf });
  (r, w)
}

impl File for Pipe {
  fn readable(&self) -> bool { !self.writable }
  fn writable(&self) -> bool { self.writable }
  fn read(&self, buf: &mut [u8]) -> usize {
    assert!(self.readable());
    let mut buf = buf.into_iter();
    let mut n = 0;
    let pipe_buf = self.buf.get();
    loop {
      if pipe_buf.buf.is_empty() {
        // All writers have closed.
        if pipe_buf.write_end.upgrade().is_none() {
          return n;
        }
        task::sched_yield();
      }
      while let Some(&x) = pipe_buf.buf.front() {
        if let Some(b) = buf.next() {
          *b = x;
          pipe_buf.buf.pop_front();
          n += 1;
        } else {
          return n;
        }
      }
    }
  }
  fn write(&self, buf: &[u8]) -> usize {
    assert!(self.writable());
    self.buf.get().buf.extend(buf.iter().copied());
    buf.len()
  }
}
