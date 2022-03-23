use crate::{*, trap::*, mm::*, fs::*};
use super::*;
use alloc::sync::Arc;

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

#[cfg(debug_assertions)]
pub const TASK_SIZE: usize = 32768;

#[cfg(not(debug_assertions))]
pub const TASK_SIZE: usize = 8192;

#[cfg(debug_assertions)]
#[repr(align(32768))]
struct TaskAlign;

#[cfg(not(debug_assertions))]
#[repr(C, align(8192))]
struct TaskAlign;

#[repr(C)]
pub struct Task {
  _align: TaskAlign,
  pub id: usize,
  pub status: TaskStatus,
  pub exit_code: i32,
  pub ctx: Context,
  pub vm: Option<MemorySet>,
  pub parent: Option<&'static mut Task>,
  pub children: Vec<&'static mut Task>,
  pub file_table: Vec<Option<Arc<dyn File>>>,
  kstack: [u8; TASK_SIZE - size_of::<usize>() * 2 - size_of::<Context>() -
    size_of::<Option<MemorySet>>() - size_of::<Option<&mut Task>>() - size_of::<Vec<&mut Task>>() -
    size_of::<Vec<Option<Arc<dyn File>>>>()],
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
    t.file_table = self.file_table.clone();
    let f1 = t.syscall_frame();
    *f1 = *f;
    f1.caller.rax = 0;
    let ret = t.id as _;
    self.add_child(&mut t);
    TASK_MANAGER.get().enqueue(t);
    ret
  }

  pub fn exec(&mut self, path: &str, f: &mut SyscallFrame) -> isize {
    if let Some(file) = open_file(path, OpenFlags::RDONLY) {
      let elf_data = file.read_all();
      let (entry, vm) = mm::load_app(&elf_data);
      self.vm = Some(vm);
      f.caller.rcx = entry;
      f.caller.r11 = x86_64::RFLAGS_IF;
      f.callee.rsp = mm::USTACK_TOP;
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

  pub fn root_pa(&self) -> PhysAddr {
    self.vm.as_ref().unwrap().pt.root_pa
  }

  pub fn add_child(&mut self, child: &mut Task) {
    unsafe {
      child.parent = transmute(self as *mut _);
      self.children.push(transmute(child));
    }
  }

  pub fn add_file(&mut self, file: Arc<dyn File>) -> usize {
    for (i, f) in self.file_table.iter_mut().enumerate() {
      if f.is_none() {
        *f = Some(file);
        return i;
      }
    }
    self.file_table.push(Some(file));
    self.file_table.len() - 1
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
    (&mut p.file_table as *mut Vec<Option<Arc<dyn File>>>).write(
      vec![Some(Arc::new(Stdin)), Some(Arc::new(Stdout)), Some(Arc::new(Stdout))]);
    t.assume_init()
  }
}

pub fn new_user(entry: usize, vm: MemorySet) -> Box<Task> {
  let mut t = new_kernel(user_task_entry, entry);
  t.vm = Some(vm);
  let f = t.syscall_frame();
  f.caller.rcx = entry;
  f.caller.r11 = x86_64::RFLAGS_IF;
  f.callee.rsp = mm::USTACK_TOP;
  t
}
