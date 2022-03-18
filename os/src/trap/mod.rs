use crate::x86_64::*;
use core::arch::global_asm;

global_asm!(include_str!("trap.S"));
global_asm!(include_str!("vector.S"));

#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct CallerRegs {
  pub rax: u64,
  pub rcx: u64,
  pub rdx: u64,
  pub rsi: u64,
  pub rdi: u64,
  pub r8: u64,
  pub r9: u64,
  pub r10: u64,
  pub r11: u64,
}

#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct CalleeRegs {
  pub rsp: u64,
  pub rbx: u64,
  pub rbp: u64,
  pub r12: u64,
  pub r13: u64,
  pub r14: u64,
  pub r15: u64,
}

#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct TrapFrame {
  pub regs: CallerRegs,
  pub id: u64,
  pub err: u64,
  // Pushed by CPU
  pub rip: u64,
  pub cs: u64,
  pub rflags: u64,
  pub ss: u64,
  pub rsp: u64,
}

pub const KCODE_SEL: u16 = (1 << 3) | RING0 as u16;
pub const KDATA_SEL: u16 = (2 << 3) | RING0 as u16;
pub const UDATA_SEL: u16 = (3 << 3) | RING3 as u16;
pub const UCODE_SEL: u16 = (4 << 3) | RING3 as u16;

extern "C" {
  static __vectors: [u64; 256];
  fn syscall_entry();
  pub fn syscall_return(ctx: &CallerRegs) -> !;
}

#[no_mangle]
pub static mut TSS: [u32; 26] = [0; 26];

#[inline]
pub fn set_user_rsp(rsp: u64) {
  unsafe {
    (TSS.as_ptr().add(3) as *mut u64).write(rsp);
  }
}

pub fn init() {
  static mut GDT: [u64; 7] = [
    0,
    0x00209800_00000000, // KCODE, EXECUTABLE | USER_SEGMENT | PRESENT | LONG_MODE
    0x00009200_00000000, // KDATA, DATA_WRITABLE | USER_SEGMENT | PRESENT
    0x0000F200_00000000, // UDATA, DATA_WRITABLE | USER_SEGMENT | USER_MODE | PRESENT
    0x0020F800_00000000, // UCODE, EXECUTABLE | USER_SEGMENT | USER_MODE | PRESENT | LONG_MODE
    0, 0, // TSS, filled in runtime
  ];

  unsafe {
    let ptr = TSS.as_ptr() as u64;
    let low = (1 << 47) | 0b1001 << 40 | (core::mem::size_of_val(&TSS) as u64 - 1) |
      ((ptr & ((1 << 24) - 1)) << 16) | (((ptr >> 24) & ((1 << 8) - 1)) << 56);
    let high = ptr >> 32;
    GDT[5] = low;
    GDT[6] = high;
    lgdt(&DescriptorTablePointer { limit: core::mem::size_of_val(&GDT) as u16 - 1, base: GDT.as_ptr() as _ });
    write_msr(KERNEL_GSBASE_MSR, TSS.as_ptr() as _);
  }
  load_tss((5 << 3) | RING0 as u16);
  write_msr(EFER_MSR, read_msr(EFER_MSR) | 1); // enable system call extensions
  write_msr(STAR_MSR, (2 << 3 << 48) | (1 << 3 << 32));
  write_msr(LSTAR_MSR, syscall_entry as usize as u64);
  write_msr(SFMASK_MSR, 0x47700); // TF|DF|IF|IOPL|AC|NT

  #[repr(C)]
  #[repr(align(16))]
  struct IDT {
    entries: [[u64; 2]; 256],
  }

  static mut IDT: IDT = unsafe { core::mem::MaybeUninit::zeroed().assume_init() };

  unsafe {
    for i in 0..256 {
      let p = __vectors[i];
      let low = (((p >> 16) & 0xFFFF) << 48) | (0b1000_1110_0000_0000 << 32) | (KCODE_SEL as u64 << 16) | (p & 0xFFFF);
      let high = p >> 32;
      IDT.entries[i] = [low, high];
    }
    lidt(&DescriptorTablePointer { limit: core::mem::size_of_val(&IDT) as u16 - 1, base: &IDT as *const _ as _ })
  }
}

#[no_mangle]
pub extern "C" fn syscall_handler(f: &'static mut CallerRegs) -> isize {
  crate::syscall::syscall(f.rax as _, [f.rdi as _, f.rsi as _, f.rdx as _])
}

#[no_mangle]
pub extern "C" fn trap_handler(f: &'static mut TrapFrame) -> isize {
  println!("trap {:x?}", f);
  crate::batch::run_next_app()
}
