mod manager;
mod signal;
mod task;

pub use signal::*;

use crate::*;
use manager::TaskManager;
use task::*;
use alloc::collections::BTreeMap;

// Dirty hack. rustc is unhappy about zero value in VecDeque.
static TASK_MANAGER: Cell<TaskManager> = unsafe { transmute([1u8; size_of::<TaskManager>()]) };
static ROOT_TASK: Cell<usize> = zero();
pub static PID2TASK: Cell<BTreeMap<usize, &'static mut Task>> = Cell::new(BTreeMap::new());

pub fn init() -> ! {
  assert_eq!(size_of::<Task>(), TASK_SIZE);
  let mut m = TaskManager::new();
  m.enqueue(Task::new(|_| {
    let cur = current();
    // Running idle and recycle orphans.
    loop {
      x86_64::disable_interrupts();
      cur.waitpid(-1);
      x86_64::enable_interrupts_and_hlt();
    }
  }, 0));
  // m.enqueue(Task::new(|arg| {
  //   println!("test kernel task 0: arg = {:#x}", arg);
  //   0
  // }, 0xdead));
  // m.enqueue(Task::new(|arg| {
  //   println!("test kernel task 1: arg = {:#x}", arg);
  //   0
  // }, 0xbeef));
  let mut shell = Task::new(user_task_entry, 0);
  shell.exec("user_shell", Vec::new());
  m.enqueue(shell);
  // let (entry, vm) = mm::load_app(&fs::open_file("user_shell", fs::OpenFlags::RDONLY).unwrap().read_all());
  // m.enqueue(new_user(entry, vm));
  let root = m.dequeue();
  unsafe {
    *ROOT_TASK.get() = root as *mut _ as _;
    (TASK_MANAGER.get() as *mut TaskManager).write(m);
    context_switch(&mut Context::default(), &root.ctx);
  }
  unreachable!();
}

pub fn root_task() -> &'static mut Task {
  unsafe { transmute(*ROOT_TASK) }
}

pub fn current() -> &'static mut Task {
  unsafe { &mut *((x86_64::read_rsp() & !(TASK_SIZE - 1)) as *mut _) }
}

pub fn sched_yield() {
  TASK_MANAGER.get().resched();
}
