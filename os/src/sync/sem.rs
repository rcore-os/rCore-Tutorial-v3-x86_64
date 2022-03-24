use crate::{*, task::*};

pub struct Sem {
  pub n: Cell<isize>,
  pub wait_queue: Cell<VecDeque<TaskPtr>>,
}

impl Sem {
  pub fn new(n: usize) -> Self {
    Self { n: Cell::new(n as _), wait_queue: Cell::default() }
  }

  pub fn up(&self) {
    *self.n.get() += 1;
    if *self.n <= 0 {
      if let Some(t) = self.wait_queue.get().pop_front() {
        task::sched_unblock(t);
      }
    }
  }

  pub fn down(&self) {
    *self.n.get() -= 1;
    if *self.n < 0 {
      self.wait_queue.get().push_back(task::current());
      task::sched_block();
    }
  }
}
