.data
.align 4
TSS:
  .space 104

.text

.macro save
  push r11
  push r10
  push r9
  push r8
  push rdi
  push rsi
  push rdx
  push rcx
  push rax
.endm

.macro restore
  pop rax
  pop rcx
  pop rdx
  pop rsi
  pop rdi
  pop r8
  pop r9
  pop r10
  pop r11
.endm

.global __trap_entry
__trap_entry:
  save
  mov rdi, rsp
  call trap_handler
  mov rax, [rsp + 96] # 96 = offsetof(TrapFrame, cs)
  and rax, 0x3
  jz __from_kernel
  lea rax, [rsp + 128] # prepare new TSS.sp0, 128 = sizeof(TrapFrame)
  mov [TSS + rip + 4], rax
__from_kernel:
  restore
  add rsp, 16 # skip TrapFrame.err and id
  iretq

.global syscall_entry
syscall_entry:
  # syscall instruction do:
  # - load cs, ss from STAR MSR
  # - r11 <- rflags, mask rflags from RFMASK MSR
  # - rcx <- rip, load rip from LSTAR MSR
  mov [TSS + rip + 12], rsp # temporarily store user rsp -> scratch at TSS.sp1
  mov rsp, [TSS + rip + 4] # load kernel rsp <- TSS.sp0
  push [TSS + rip + 12] # store user rsp -> SyscallFrame.rsp
  save
  mov rdi, rsp
  call syscall_handler
  mov [rsp], rax # CallerRegs.rax is at offset 0
  jmp __syscall_return

.global syscall_return
syscall_return: # (CallerRegs *)
  mov rsp, rdi
__syscall_return:
  lea rax, [rsp + 80] # prepare new TSS.sp0, 80 = sizeof(SyscallFrame)
  mov [TSS + rip + 4], rax
  restore
  pop rsp # restore user rsp
  sysretq
