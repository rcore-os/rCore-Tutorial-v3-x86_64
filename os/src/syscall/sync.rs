use crate::{*, sync::*};

pub fn sys_sleep(ms: usize) -> isize {
  task::add_timer(ms);
  0
}

pub fn sys_mutex_create(blocking: bool) -> isize {
  let p = &mut task::current().proc;
  let mutex: Box<dyn Mutex> = if blocking {
    Box::new(MutexBlocking::default())
  } else {
    Box::new(MutexSpin::default())
  };
  p.mutexes.push(mutex);
  (p.mutexes.len() - 1) as _
}

pub fn sys_mutex_lock(mutex_id: usize) -> isize {
  let p = &mut task::current().proc;
  try_!(p.mutexes.get(mutex_id), -1).lock();
  0
}

pub fn sys_mutex_unlock(mutex_id: usize) -> isize {
  let p = &mut task::current().proc;
  try_!(p.mutexes.get(mutex_id), -1).unlock();
  0
}

pub fn sys_semaphore_create(n: usize) -> isize {
  let p = &mut task::current().proc;
  p.sems.push(Sem::new(n));
  (p.sems.len() - 1) as _
}

pub fn sys_semaphore_up(sem_id: usize) -> isize {
  let p = &mut task::current().proc;
  try_!(p.sems.get(sem_id), -1).up();
  0
}

pub fn sys_semaphore_down(sem_id: usize) -> isize {
  let p = &mut task::current().proc;
  try_!(p.sems.get(sem_id), -1).down();
  0
}

pub fn sys_condvar_create() -> isize {
  let p = &mut task::current().proc;
  p.condvars.push(Condvar::default());
  (p.condvars.len() - 1) as _
}

pub fn sys_condvar_signal(condvar_id: usize) -> isize {
  let p = &mut task::current().proc;
  try_!(p.condvars.get(condvar_id), -1).signal();
  0
}

pub fn sys_condvar_wait(condvar_id: usize, mutex_id: usize) -> isize {
  let p = &mut task::current().proc;
  let condvar = try_!(p.condvars.get(condvar_id), -1);
  let mutex = try_!(p.mutexes.get(mutex_id), -1);
  condvar.wait(mutex.as_ref());
  0
}
