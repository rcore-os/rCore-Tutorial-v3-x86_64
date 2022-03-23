mod inode;
mod pipe;
mod stdio;

pub trait File {
  fn readable(&self) -> bool;
  fn writable(&self) -> bool;
  fn read(&self, buf: &mut [u8]) -> usize;
  fn write(&self, buf: &[u8]) -> usize;
}

pub use inode::{init, open_file, OSInode, OpenFlags};
pub use pipe::{make_pipe, Pipe};
pub use stdio::{Stdin, Stdout};
