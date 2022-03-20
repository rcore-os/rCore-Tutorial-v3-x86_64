use crate::*;
use super::*;

pub struct TaskManager {
  pub tasks: Vec<Box<Task>>,
}

impl TaskManager {
  fn pick_next_task(&mut self) -> &mut Task {
    let cur = current();
    let start = if cur.status == TaskStatus::UnInit { 0 } else { cur.id + 1 };
    let mut i = 0;
    let n = self.tasks.len();
    loop {
      let id = start + i;
      let id = if id >= n { id - n } else { id };
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
      cur.switch_to(nxt);
    }
  }
}
