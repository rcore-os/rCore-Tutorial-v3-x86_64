use crate::{*, trap::*, mm::*};
use super::*;
// use alloc::sync::{Arc, Weak};
// use alloc::rc::{Rc, Weak};

core::arch::global_asm!(include_str!("switch.S"));

#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct Context {
  pub regs: CalleeRegs,
  pub rip: usize,
}

extern "C" {
  pub fn context_switch(cur: &mut Context, nxt: &Context);
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(i32)]
pub enum TaskStatus {
  Runnable,
  Zombie,
}

pub const TASK_SIZE: usize = 8192;

#[repr(C, align(8192))] // TASK_SIZE
pub struct Task {
  pub id: usize,
  pub status: TaskStatus,
  pub exit_code: i32,
  pub ctx: Context,
  pub vm: Option<MemorySet>,
  pub parent: Option<&'static mut Task>,
  pub children: Vec<&'static mut Task>,
  kstack: [u8; TASK_SIZE - size_of::<usize>() * 2 - size_of::<Context>() -
    size_of::<Option<MemorySet>>() - size_of::<Option<&mut Task>>() - size_of::<Vec<&mut Task>>()],
}

fn new_id() -> usize {
  static NEXT_ID: Cell<usize> = zero();
  let next = *NEXT_ID + 1;
  *NEXT_ID.get() = next;
  next
}

fn user_task_entry(_: usize) -> usize {
  unsafe { syscall_return(current().syscall_frame()); }
}

impl Task {
  pub fn exit(&mut self, exit_code: i32) -> ! {
    println!("[kernel] Task {} exited with code {}", self.id, exit_code);
    self.vm = None;
    self.exit_code = exit_code;
    self.status = TaskStatus::Zombie;
    for ch in &mut self.children {
      root_task().add_child(ch);
    }
    self.children.clear();
    TASK_MANAGER.get().resched();
    unreachable!("task exited!");
  }

  pub fn fork(&mut self, f: &SyscallFrame) -> isize {
    let mut t = new_kernel(user_task_entry, 0);
    t.vm = self.vm.clone();
    let f1 = t.syscall_frame();
    *f1 = *f;
    f1.caller.rax = 0;
    let ret = t.id as _;
    self.add_child(&mut t);
    TASK_MANAGER.get().enqueue(t);
    ret
  }

  pub fn exec(&mut self, path: &str, f: &mut SyscallFrame) -> isize {
    if let Some(elf_data) = loader::get_app_data_by_name(path) {
      let (entry, vm) = loader::load_app(elf_data);
      self.vm = Some(vm);
      f.caller.rcx = entry;
      f.caller.r11 = x86_64::RFLAGS_IF;
      f.callee.rsp = loader::USTACK_TOP;
      0
    } else {
      -1
    }
  }

  pub fn waitpid(&mut self, pid: isize) -> (isize, i32) {
    let mut found_pid = false;
    for (idx, t) in self.children.iter().enumerate() {
      if pid == -1 || t.id == pid as usize {
        found_pid = true;
        if t.status == TaskStatus::Zombie {
          let child = self.children.remove(idx);
          let ret = (child.id as _, child.exit_code);
          unsafe { Box::from_raw(child); } // Drop it.
          return ret;
        }
      }
    }
    (if found_pid { -2 } else { -1 }, 0)
  }

  pub fn syscall_frame(&mut self) -> &mut SyscallFrame {
    unsafe { &mut *(self.kstack.as_ptr_range().end as *mut SyscallFrame).sub(1) }
  }

  pub fn switch_to(&mut self, nxt: &Task) {
    if let Some(vm) = &nxt.vm {
      vm.activate(); // user task
    }
    unsafe { context_switch(&mut self.ctx, &nxt.ctx); }
  }

  pub fn add_child(&mut self, child: &mut Task) {
    unsafe {
      child.parent = transmute(self as *mut _);
      self.children.push(transmute(child));
    }
  }
}

pub fn new_kernel(entry: fn(usize) -> usize, arg: usize) -> Box<Task> {
  fn kernel_task_entry() -> ! {
    let cur = current();
    let entry: fn(usize) -> usize = unsafe { transmute(cur.ctx.regs.rbx) };
    let arg = cur.ctx.regs.rbp;
    let ret = entry(arg);
    cur.exit(ret as _);
  }
  let mut t = Box::<Task>::new_uninit();
  let p = unsafe { &mut *t.as_mut_ptr() };
  p.id = new_id();
  p.status = TaskStatus::Runnable;
  p.ctx.rip = kernel_task_entry as _;
  p.ctx.regs.rsp = p.kstack.as_ptr_range().end as usize - size_of::<usize>() - size_of::<SyscallFrame>();
  p.ctx.regs.rbx = entry as _;
  p.ctx.regs.rbp = arg;
  p.parent = None;
  unsafe {
    (&mut p.vm as *mut Option<MemorySet>).write(None);
    (&mut p.children as *mut Vec<&mut Task>).write(Vec::new());
    t.assume_init()
  }
}

pub fn new_user(entry: usize, vm: MemorySet) -> Box<Task> {
  let mut t = new_kernel(user_task_entry, entry);
  t.vm = Some(vm);
  let f = t.syscall_frame();
  f.caller.rcx = entry;
  f.caller.r11 = x86_64::RFLAGS_IF;
  f.callee.rsp = loader::USTACK_TOP;
  t
}
