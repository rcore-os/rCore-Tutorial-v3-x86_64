mod manager;

use crate::{*, trap::*};

core::arch::global_asm!(include_str!("switch.S"));

static TASK_MANAGER: Cell<manager::TaskManager> = zero();

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
  kstack: [u8; TASK_SIZE - 10 * 8],
}

impl Task {
  pub fn init_kernel(&mut self, id: usize, entry: fn(usize) -> usize, arg: usize) {
    fn kernel_task_entry() -> ! {
      let cur = current();
      let entry: fn(usize) -> usize = unsafe { transmute(cur.ctx.regs.rbx) };
      let arg = cur.ctx.regs.rbp;
      let ret = entry(arg);
      current_exit(ret as _);
    }
    self.id = id;
    self.ctx.rip = kernel_task_entry as _;
    self.ctx.regs.rsp = self.kstack.as_ptr_range().end as usize - size_of::<usize>();
    self.ctx.regs.rbx = entry as _;
    self.ctx.regs.rbp = arg;
    self.status = TaskStatus::Runnable;
  }

  pub fn init_user(&mut self, id: usize, entry: usize, ustack_top: usize) {
    fn user_task_entry(_: usize) -> usize {
      let cur = current();
      let entry = cur.ctx.regs.r12;
      let ustack_top = cur.ctx.regs.r13;
      unsafe {
        let f = &mut *((cur.kstack.as_ptr_range().end as *mut SyscallFrame).sub(1));
        f.regs.rcx = entry;
        f.regs.r11 = x86_64::RFLAGS_IF;
        f.rsp = ustack_top;
        syscall_return(f);
      }
    }
    self.init_kernel(id, user_task_entry, 0);
    self.ctx.regs.r12 = entry;
    self.ctx.regs.r13 = ustack_top;
  }
}

pub fn init() -> ! {
  assert_eq!(size_of::<Task>(), TASK_SIZE);
  TASK_MANAGER.get().init();
  unsafe { context_switch(&mut Context::default(), &TASK_MANAGER.get().tasks[0].ctx); }
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
