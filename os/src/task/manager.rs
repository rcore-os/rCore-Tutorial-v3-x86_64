use crate::*;
use super::*;

pub struct TaskManager {
  task_count: usize,
  pub(crate) tasks: [Task; loader::MAX_APP_NUM],
}

impl TaskManager {
  pub fn init(&mut self) {
    let kernel_task_count = 3;
    self.task_count = loader::get_app_count() + kernel_task_count;
    self.tasks[0].init_kernel(0, |_| {
      // running idle.
      loop { x86_64::enable_and_hlt(); }
    }, 0);
    self.tasks[1].init_kernel(
      1, |arg| {
        println!("test kernel task 0: arg = {:#x}", arg);
        0
      }, 0xdead,
    );
    self.tasks[2].init_kernel(
      2, |arg| {
        println!("test kernel task 1: arg = {:#x}", arg);
        0
      }, 0xbeef,
    );
    for i in 0..self.task_count - kernel_task_count {
      let (entry, ustack_top) = loader::load_app(i);
      self.tasks[i + kernel_task_count].init_user(i + kernel_task_count, entry, ustack_top);
    }
  }

  fn pick_next_task(&mut self) -> &mut Task {
    let cur = current();
    let start = if cur.status == TaskStatus::UnInit { 0 } else { cur.id + 1 };
    let mut i = 0;
    loop {
      let id = start + i;
      let id = if id >= self.task_count { id - self.task_count } else { id };
      if self.tasks[id].status == TaskStatus::Runnable {
        if start == 1 && id == 0 {
          panic!("All applications completed! only idle task remains");
        }
        return &mut self.tasks[id];
      }
      i += 1;
    }
  }

  pub fn resched(&mut self) {
    let cur = current();
    let nxt = self.pick_next_task();
    if cur as *const Task != nxt as *const Task {
      unsafe { super::context_switch(&mut cur.ctx, &nxt.ctx); }
    }
  }
}
