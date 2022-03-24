mod manager;
mod proc;
mod signal;
mod task;

pub use self::{manager::*, proc::*, signal::*, task::*};

use crate::{*, fs::*};

// Dirty hack. rustc is unhappy about zero value in VecDeque.
static TASK_MANAGER: Cell<TaskManager> = unsafe { transmute([1u8; size_of::<TaskManager>()]) };
static ROOT_PROC: Cell<usize> = zero();

pub fn init() -> ! {
  assert_eq!(size_of::<Task>(), TASK_SIZE);
  unsafe { (TASK_MANAGER.get() as *mut TaskManager).write(TaskManager::default()); }
  let root = Box::leak(Box::new(Proc {
    pid: new_id(),
    files: vec![Some(Rc::new(Stdin)), Some(Rc::new(Stdout)), Some(Rc::new(Stdout))],
    ..Proc::default()
  }));
  *ROOT_PROC.get() = root as *mut _ as _;
  Task::new(root, |_| {
    let cur = current();
    // Running idle and recycle orphans.
    loop {
      x86_64::disable_interrupts();
      cur.proc.waitpid(-1);
      x86_64::enable_interrupts_and_hlt();
    }
  }, 0);
  let shell = root.fork();
  shell.exec("user_shell", Vec::new());
  unsafe { context_switch(&mut Context::default(), &TASK_MANAGER.get().dequeue().ctx); }
  unreachable!();
}

pub fn root_proc() -> ProcPtr {
  unsafe { transmute(*ROOT_PROC) }
}

pub fn current() -> TaskPtr {
  unsafe { &mut *((x86_64::read_rsp() & !(TASK_SIZE - 1)) as *mut _) }
}

pub fn sched_yield() {
  TASK_MANAGER.get().resched();
}

pub fn sched_block() {
  current().status = TaskStatus::Blocking;
  TASK_MANAGER.get().resched();
}

pub fn sched_unblock(t: &mut Task) {
  t.status = TaskStatus::Runnable;
  TASK_MANAGER.get().enqueue(t);
}
