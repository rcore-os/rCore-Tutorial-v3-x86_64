use crate::{*, mm::*, fs::*, sync::*};
use super::*;

#[derive(Default)]
pub struct Proc {
  pub pid: usize,
  pub signal: SignalFlags,
  pub zombie: bool,
  pub exit_code: i32,
  pub vm: Option<MemorySet>,
  pub parent: Option<ProcPtr>,
  pub children: Vec<ProcPtr>,
  pub tasks: Vec<Box<Task>>,
  pub files: Vec<Option<Rc<dyn File>>>,
  pub mutexes: Vec<Box<dyn Mutex>>,
  pub sems: Vec<Sem>,
  pub condvars: Vec<Condvar>,
}

pub type ProcPtr = &'static mut Proc;

pub(crate) fn new_id() -> usize {
  static NEXT_ID: Cell<usize> = zero();
  let next = *NEXT_ID + 1;
  *NEXT_ID.get() = next;
  next
}

pub static PID2PROC: Cell<BTreeMap<usize, ProcPtr>> = Cell::new(BTreeMap::new());

impl Proc {
  /// Only support processes with a single thread.
  pub fn fork(&mut self) -> ProcPtr {
    assert_eq!(self.tasks.len(), 1);
    let child = Box::leak(Box::new(Proc {
      pid: new_id(),
      vm: self.vm.clone(),
      files: self.files.clone(),
      ..Proc::default()
    }));
    let t = unsafe {
      let child = child as *mut Proc; // Escape borrow checker.
      PID2PROC.get().insert((*child).pid, &mut *child);
      self.add_child(&mut *child);
      Task::new(&mut *child, user_task_entry, 0).0
    };
    let f = t.syscall_frame();
    *f = *self.tasks[0].syscall_frame();
    f.caller.rax = 0;
    child
  }

  /// Only support processes with a single thread.
  pub fn exec(&mut self, path: &str, args: Vec<String>) -> isize {
    assert_eq!(self.tasks.len(), 1);
    if let Some(file) = open_file(path, OpenFlags::RDONLY) {
      let elf_data = file.read_all();
      let (entry, vm) = mm::load_app(&elf_data);
      vm.activate(); // To access ustack.
      let mut top = (USTACK_TOP - (args.len() + 1) * size_of::<usize>()) as *mut u8;
      let argv = top as *mut usize;
      unsafe {
        for (i, arg) in args.iter().enumerate() {
          top = top.sub(arg.len() + 1);
          core::ptr::copy_nonoverlapping(arg.as_ptr(), top, arg.len());
          *top.add(arg.len()) = 0; // '\0' terminator.
          *argv.add(i) = top as _;
        }
        // Set argv[argc] = NULL, some C programs rely on this.
        *argv.add(args.len()) = 0;
      }
      self.vm = Some(vm);
      let f = self.tasks[0].syscall_frame();
      f.caller.rcx = entry;
      f.caller.r11 = x86_64::RFLAGS_IF;
      f.callee.rsp = top as usize & !0xF; // Align down to 16.
      f.caller.rdi = args.len(); // _start parameter argc.
      f.caller.rsi = argv as _; // _start parameter argv.
      0
    } else {
      -1
    }
  }

  pub fn waitpid(&mut self, pid: isize) -> (isize, i32) {
    let mut found_pid = false;
    for (idx, p) in self.children.iter().enumerate() {
      if pid == -1 || p.pid == pid as usize {
        found_pid = true;
        if p.zombie {
          let child = self.children.remove(idx);
          let ret = (child.pid as _, child.exit_code);
          unsafe { Box::from_raw(child); } // Drop it.
          return ret;
        }
      }
    }
    (if found_pid { -2 } else { -1 }, 0)
  }

  pub fn root_pa(&self) -> PhysAddr {
    self.vm.as_ref().unwrap().pt.root_pa
  }

  pub fn add_child(&mut self, child: &mut Proc) {
    unsafe {
      child.parent = transmute(self as *mut _);
      self.children.push(transmute(child));
    }
  }

  pub fn add_file(&mut self, file: Rc<dyn File>) -> usize {
    for (i, f) in self.files.iter_mut().enumerate() {
      if f.is_none() {
        *f = Some(file);
        return i;
      }
    }
    self.files.push(Some(file));
    self.files.len() - 1
  }

  pub fn add_signal(&mut self, signal: SignalFlags) {
    assert!(self.vm.is_some()); // Must not be a kernel task.
    self.signal |= signal;
  }
}
