use crate::{*, task::*};
use super::*;

#[derive(Default)]
pub struct Condvar {
  pub wait_queue: Cell<VecDeque<TaskPtr>>,
}

impl Condvar {
  pub fn signal(&self) {
    if let Some(t) = self.wait_queue.get().pop_front() {
      task::sched_unblock(t);
    }
  }

  pub fn wait(&self, mutex: &dyn Mutex) {
    mutex.unlock();
    self.wait_queue.get().push_back(task::current());
    task::sched_block();
    mutex.lock();
  }
}
