use crate::*;
use super::*;
use alloc::collections::BinaryHeap;

#[derive(Default)]
pub struct TaskManager {
  runnable: VecDeque<TaskPtr>,
}

impl TaskManager {
  pub fn enqueue(&mut self, t: &mut Task) {
    self.runnable.push_back(unsafe { transmute(t) });
  }

  pub fn dequeue(&mut self) -> TaskPtr {
    self.runnable.pop_front().unwrap()
  }

  pub fn clear_zombie(&mut self) {
    let len = self.runnable.len();
    for _ in 0..len {
      let t = self.runnable.pop_front().unwrap();
      if t.status == TaskStatus::Runnable {
        self.runnable.push_back(t);
      }
    }
  }

  pub fn resched(&mut self) {
    let cur = current();
    if cur.status == TaskStatus::Runnable {
      self.enqueue(cur);
    }
    let nxt = self.dequeue();
    if cur as *const _ != nxt as *const _ {
      cur.switch_to(nxt);
    }
  }
}

struct SleepingTask {
  expire_ms: usize,
  task: TaskPtr,
}

impl PartialEq for SleepingTask {
  fn eq(&self, other: &Self) -> bool { self.expire_ms == other.expire_ms }
}

impl Eq for SleepingTask {}

impl PartialOrd for SleepingTask {
  fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for SleepingTask {
  fn cmp(&self, other: &Self) -> core::cmp::Ordering {
    self.expire_ms.cmp(&other.expire_ms).reverse()
  }
}

// Currently BinaryHeap::new is equivalent to Vec::new.
static TIMERS: Cell<BinaryHeap<SleepingTask>> = unsafe { transmute(Vec::<SleepingTask>::new()) };

pub fn add_timer(ms: usize) {
  TIMERS.get().push(SleepingTask { expire_ms: *pic::TICKS + ms, task: current() });
  self::sched_block();
}

pub fn clear_zombie_timer() {
  let timers = core::mem::replace(TIMERS.get(), BinaryHeap::new());
  for t in timers {
    if t.task.status == TaskStatus::Runnable {
      TIMERS.get().push(t);
    }
  }
}

pub fn check_timer() {
  let current_ms = *pic::TICKS;
  while let Some(t) = TIMERS.get().peek() {
    if t.expire_ms <= current_ms {
      self::sched_unblock(unsafe { &mut *(t.task as *const _ as *mut _) });
      TIMERS.get().pop();
    } else {
      break;
    }
  }
}
