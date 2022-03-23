use crate::task::current;
bitflags::bitflags! {
  pub struct SignalFlags: u16 {
    const SIGINT    = 1 << 2;
    const SIGILL    = 1 << 4;
    const SIGABRT   = 1 << 6;
    const SIGFPE    = 1 << 8;
    const SIGSEGV   = 1 << 11;
  }
}

impl SignalFlags {
  pub fn check_error(self) -> Option<(i32, &'static str)> {
    if self.contains(Self::SIGINT) {
      Some((-2, "Killed, SIGINT=2"))
    } else if self.contains(Self::SIGILL) {
      Some((-4, "Illegal Instruction, SIGILL=4"))
    } else if self.contains(Self::SIGABRT) {
      Some((-6, "Aborted, SIGABRT=6"))
    } else if self.contains(Self::SIGFPE) {
      Some((-8, "Erroneous Arithmetic Operation, SIGFPE=8"))
    } else if self.contains(Self::SIGSEGV) {
      Some((-11, "Segmentation Fault, SIGSEGV=11"))
    } else {
      None
    }
  }
}

pub fn current_check_signal() {
  let t = current();
  if let Some((code, msg)) = t.signal.check_error() {
    println!("[kernel] {}", msg);
    t.exit(code);
  }
}
