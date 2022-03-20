mod manager;

use crate::{*, trap::*, mm::*};
use manager::TaskManager;

core::arch::global_asm!(include_str!("switch.S"));

static TASK_MANAGER: Cell<TaskManager> = Cell::new(TaskManager { tasks: Vec::new() });

#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct Context {
  pub regs: CalleeRegs,
  pub rip: usize,
}

extern "C" {
  fn context_switch(cur: &mut Context, nxt: &Context);
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(usize)]
pub enum TaskStatus {
  UnInit = 0,
  Runnable,
  Exited,
}

const TASK_SIZE: usize = 8192;

#[repr(C, align(8192))] // TASK_SIZE
pub struct Task {
  id: usize,
  status: TaskStatus,
  ctx: Context,
  memory_set: Option<MemorySet>,
  kstack: [u8; TASK_SIZE - size_of::<usize>() - size_of::<TaskStatus>() -
    size_of::<Context>() - size_of::<Option<MemorySet>>()],
}

impl Task {
  pub fn new_kernel(id: usize, entry: fn(usize) -> usize, arg: usize) -> Box<Self> {
    fn kernel_task_entry() -> ! {
      let cur = current();
      let entry: fn(usize) -> usize = unsafe { transmute(cur.ctx.regs.rbx) };
      let arg = cur.ctx.regs.rbp;
      let ret = entry(arg);
      current_exit(ret as _);
    }
    let mut t = Box::<Task>::new_uninit();
    let p = unsafe { &mut *t.as_mut_ptr() };
    p.id = id;
    p.status = TaskStatus::Runnable;
    p.ctx.rip = kernel_task_entry as _;
    p.ctx.regs.rsp = p.kstack.as_ptr_range().end as usize - size_of::<usize>();
    p.ctx.regs.rbx = entry as _;
    p.ctx.regs.rbp = arg;
    unsafe {
      (&mut p.memory_set as *mut Option<MemorySet>).write(None);
      t.assume_init()
    }
  }

  pub fn new_user(id: usize, entry: usize, ms: MemorySet) -> Box<Self> {
    fn user_task_entry(entry: usize) -> usize {
      let cur = current();
      unsafe {
        let f = &mut *((cur.kstack.as_ptr_range().end as *mut SyscallFrame).sub(1));
        f.regs.rcx = entry;
        f.regs.r11 = x86_64::RFLAGS_IF;
        f.rsp = loader::USTACK_TOP;
        syscall_return(f);
      }
    }
    let mut t = Self::new_kernel(id, user_task_entry, entry);
    t.memory_set = Some(ms);
    t
  }

  fn switch_to(&mut self, nxt: &Task) {
    if let Some(ms) = &nxt.memory_set {
      ms.activate(); // user task
    }
    unsafe { context_switch(&mut self.ctx, &nxt.ctx); }
  }
}

pub fn init() -> ! {
  assert_eq!(size_of::<Task>(), TASK_SIZE);
  let m = TASK_MANAGER.get();
  let kernel_task_count = 3;
  m.tasks.push(Task::new_kernel(0, |_| {
    // running idle.
    loop { x86_64::enable_and_hlt(); }
  }, 0));
  m.tasks.push(Task::new_kernel(1, |arg| {
    println!("test kernel task 0: arg = {:#x}", arg);
    0
  }, 0xdead));
  m.tasks.push(Task::new_kernel(2, |arg| {
    println!("test kernel task 1: arg = {:#x}", arg);
    0
  }, 0xbeef));
  for i in 0..loader::get_app_count() {
    let (entry, ms) = loader::load_app(i);
    m.tasks.push(Task::new_user(i + kernel_task_count, entry, ms));
  }
  unsafe { context_switch(&mut Context::default(), &m.tasks[0].ctx); }
  unreachable!();
}

pub fn current() -> &'static mut Task {
  unsafe { &mut *((x86_64::read_rsp() & !(TASK_SIZE - 1)) as *mut Task) }
}

pub fn current_yield() {
  TASK_MANAGER.get().resched();
}

pub fn current_exit(exit_code: i32) -> ! {
  let cur = current();
  println!("[kernel] Task {} exited with code {}", cur.id, exit_code);
  cur.status = TaskStatus::Exited;
  TASK_MANAGER.get().resched();
  unreachable!("task exited!");
}
