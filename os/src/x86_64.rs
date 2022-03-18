use core::arch::asm;

pub const RING0: u64 = 0;
pub const RING3: u64 = 3;

// bitflags! {
//     /// The RFLAGS register.
//     pub struct RFlags: u64 {
//         /// Processor feature identification flag.
//         ///
//         /// If this flag is modifiable, the CPU supports CPUID.
//         const ID = 1 << 21;
//         /// Indicates that an external, maskable interrupt is pending.
//         ///
//         /// Used when virtual-8086 mode extensions (CR4.VME) or protected-mode virtual
//         /// interrupts (CR4.PVI) are activated.
//         const VIRTUAL_INTERRUPT_PENDING = 1 << 20;
//         /// Virtual image of the INTERRUPT_FLAG bit.
//         ///
//         /// Used when virtual-8086 mode extensions (CR4.VME) or protected-mode virtual
//         /// interrupts (CR4.PVI) are activated.
//         const VIRTUAL_INTERRUPT = 1 << 19;
//         /// Enable automatic alignment checking if CR0.AM is set. Only works if CPL is 3.
//         const ALIGNMENT_CHECK = 1 << 18;
//         /// Enable the virtual-8086 mode.
//         const VIRTUAL_8086_MODE = 1 << 17;
//         /// Allows to restart an instruction following an instrucion breakpoint.
//         const RESUME_FLAG = 1 << 16;
//         /// Used by `iret` in hardware task switch mode to determine if current task is nested.
//         const NESTED_TASK = 1 << 14;
//         /// The high bit of the I/O Privilege Level field.
//         ///
//         /// Specifies the privilege level required for executing I/O address-space instructions.
//         const IOPL_HIGH = 1 << 13;
//         /// The low bit of the I/O Privilege Level field.
//         ///
//         /// Specifies the privilege level required for executing I/O address-space instructions.
//         const IOPL_LOW = 1 << 12;
//         /// Set by hardware to indicate that the sign bit of the result of the last signed integer
//         /// operation differs from the source operands.
//         const OVERFLOW_FLAG = 1 << 11;
//         /// Determines the order in which strings are processed.
//         const DIRECTION_FLAG = 1 << 10;
//         /// Enable interrupts.
//         const INTERRUPT_FLAG = 1 << 9;
//         /// Enable single-step mode for debugging.
//         const TRAP_FLAG = 1 << 8;
//         /// Set by hardware if last arithmetic operation resulted in a negative value.
//         const SIGN_FLAG = 1 << 7;
//         /// Set by hardware if last arithmetic operation resulted in a zero value.
//         const ZERO_FLAG = 1 << 6;
//         /// Set by hardware if last arithmetic operation generated a carry ouf of bit 3 of the
//         /// result.
//         const AUXILIARY_CARRY_FLAG = 1 << 4;
//         /// Set by hardware if last result has an even number of 1 bits (only for some operations).
//         const PARITY_FLAG = 1 << 2;
//         /// Set by hardware if last arithmetic operation generated a carry out of the
//         /// most-significant bit of the result.
//         const CARRY_FLAG = 1;
//     }
// }

const RFLAGS_IF: u64 = 1 << 9;

#[inline]
pub fn are_enabled() -> bool {
  (read_raw() & RFLAGS_IF) != 0
}

#[inline]
pub fn enable() {
  unsafe { asm!("sti", options(nomem, nostack)); }
}

#[inline]
pub fn disable() {
  unsafe { asm!("cli", options(nomem, nostack)); }
}

#[inline]
pub fn read_raw() -> u64 {
  let r: u64;
  unsafe { asm!("pushfq; pop {}", out(reg) r, options(nomem, preserves_flags)); }
  r
}

#[inline]
pub fn write_raw(val: u64) {
  unsafe { asm!("push {}; popfq", in(reg) val, options(nomem, preserves_flags)); }
}

#[inline]
pub fn read_msr(id: u32) -> u64 {
  let (high, low): (u32, u32);
  unsafe { asm!("rdmsr", in("ecx") id, out("eax") low, out("edx") high, options(nomem, nostack, preserves_flags)); }
  ((high as u64) << 32) | (low as u64)
}

#[inline]
pub fn write_msr(id: u32, val: u64) {
  let low = val as u32;
  let high = (val >> 32) as u32;
  unsafe { asm!("wrmsr", in("ecx") id, in("eax") low, in("edx") high, options(nostack, preserves_flags)); }
}

pub const EFER_MSR: u32 = 0xC000_0080;
pub const GSBASE_MSR: u32 = 0xC000_0101;
pub const KERNEL_GSBASE_MSR: u32 = 0xC000_0102;
pub const STAR_MSR: u32 = 0xC000_0081;
pub const LSTAR_MSR: u32 = 0xC000_0082;
pub const SFMASK_MSR: u32 = 0xC000_0084;


#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct DescriptorTablePointer {
  /// Size of the DT.
  pub limit: u16,
  /// Pointer to the memory region containing the DT.
  pub base: u64,
}

/// Load a GDT.
#[inline]
pub fn lgdt(gdt: &DescriptorTablePointer) {
  unsafe { asm!("lgdt [{}]", in(reg) gdt, options(readonly, nostack, preserves_flags)); }
}

/// Load an IDT.
#[inline]
pub fn lidt(idt: &DescriptorTablePointer) {
  unsafe { asm!("lidt [{}]", in(reg) idt, options(readonly, nostack, preserves_flags)); }
}

/// Load the task state register using the `ltr` instruction.
#[inline]
pub fn load_tss(sel: u16) {
  unsafe { asm!("ltr {0:x}", in(reg) sel, options(nomem, nostack, preserves_flags)); }
}
