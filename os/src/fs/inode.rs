use crate::{*, drivers::BLOCK_DEVICE};
use super::File;

use alloc::sync::Arc;
use easy_fs::{EasyFileSystem, Inode};

pub struct OSInode {
  readable: bool,
  writable: bool,
  offset: Cell<usize>,
  inode: Cell<Arc<Inode>>,
}

pub static ROOT_INODE: Cell<Arc<Inode>> = unsafe { transmute([1u8; size_of::<Arc<Inode>>()]) };

pub fn init() {
  let efs = EasyFileSystem::open(BLOCK_DEVICE.clone());
  unsafe { (ROOT_INODE.get() as *mut Arc<Inode>).write(Arc::new(EasyFileSystem::root_inode(&efs))); }
  println!("/**** APPS ****");
  for app in ROOT_INODE.ls() {
    println!("{}", app);
  }
  println!("**************/");
}

impl OSInode {
  pub fn new(readable: bool, writable: bool, inode: Arc<Inode>) -> Self {
    Self { readable, writable, offset: Cell::new(0), inode: Cell::new(inode) }
  }

  pub fn read_all(&self) -> Vec<u8> {
    let (offset, inode) = (self.offset.get(), self.inode.get());
    let mut buffer = [0u8; 512];
    let mut v: Vec<u8> = Vec::new();
    loop {
      let len = inode.read_at(*offset, &mut buffer);
      if len == 0 { break; }
      *offset += len;
      v.extend_from_slice(&buffer[..len]);
    }
    v
  }
}

bitflags::bitflags! {
  pub struct OpenFlags: u32 {
    const RDONLY = 0;
    const WRONLY = 1 << 0;
    const RDWR = 1 << 1;
    const CREATE = 1 << 9;
    const TRUNC = 1 << 10;
  }
}

impl OpenFlags {
  /// Do not check validity for simplicity
  /// Return (readable, writable)
  pub fn read_write(&self) -> (bool, bool) {
    if self.is_empty() {
      (true, false)
    } else if self.contains(Self::WRONLY) {
      (false, true)
    } else {
      (true, true)
    }
  }
}

pub fn open_file(name: &str, flags: OpenFlags) -> Option<Arc<OSInode>> {
  let (readable, writable) = flags.read_write();
  if flags.contains(OpenFlags::CREATE) {
    if let Some(inode) = ROOT_INODE.find(name) {
      // clear size
      inode.clear();
      Some(Arc::new(OSInode::new(readable, writable, inode)))
    } else {
      // create file
      ROOT_INODE.create(name)
        .map(|inode| Arc::new(OSInode::new(readable, writable, inode)))
    }
  } else {
    ROOT_INODE.find(name).map(|inode| {
      if flags.contains(OpenFlags::TRUNC) {
        inode.clear();
      }
      Arc::new(OSInode::new(readable, writable, inode))
    })
  }
}

impl File for OSInode {
  fn readable(&self) -> bool { self.readable }
  fn writable(&self) -> bool { self.writable }
  fn read(&self, buf: &mut [u8]) -> usize {
    let (offset, inode) = (self.offset.get(), self.inode.get());
    let n = inode.read_at(*offset, buf);
    *offset += n;
    n
  }
  fn write(&self, buf: &[u8]) -> usize {
    let (offset, inode) = (self.offset.get(), self.inode.get());
    let n = inode.write_at(*offset, buf);
    assert_eq!(n, buf.len());
    *offset += n;
    n
  }
}
