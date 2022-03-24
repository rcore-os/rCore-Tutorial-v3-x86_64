use crate::{*, task::*};

pub trait Mutex {
  fn lock(&self);
  fn unlock(&self);
}

#[derive(Default)]
pub struct MutexSpin {
  locked: Cell<bool>,
}

impl Mutex for MutexSpin {
  fn lock(&self) {
    loop {
      if *self.locked {
        task::sched_yield();
      } else {
        *self.locked.get() = true;
        return;
      }
    }
  }

  fn unlock(&self) {
    *self.locked.get() = false;
  }
}

#[derive(Default)]
pub struct MutexBlocking {
  locked: Cell<bool>,
   wait_queue: Cell<VecDeque<TaskPtr>>,
}

impl Mutex for MutexBlocking {
  fn lock(&self) {
    if *self.locked {
      self.wait_queue.get().push_back(task::current());
      task::sched_block();
    } else {
      *self.locked.get() = true;
    }
  }

  fn unlock(&self) {
    if let Some(t) = self.wait_queue.get().pop_front() {
      task::sched_unblock(t);
    } else {
      *self.locked.get() = false;
    }
  }
}
