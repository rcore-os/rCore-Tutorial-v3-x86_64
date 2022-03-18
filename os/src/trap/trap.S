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
  mov [rsp], rax # TrapFrame.regs.rax is at offset 0
  mov rax, [rsp + 96] # 96 = offsetof(TrapFrame, cs)
  and rax, 0x3
  jz __from_kernel
  swapgs
  lea rax, [rax + 128] # prepare new TSS.sp0, 128 = sizeof(TrapFrame)
  mov gs:4, rax
  swapgs
__from_kernel:
  restore
  iretq

.global syscall_entry
syscall_entry:
  # syscall instruction do:
  # - load cs, ss from STAR MSR
  # - r11 <- rflags, mask rflags from RFMASK MSR
  # - rcx <- rip, load rip from LSTAR MSR
  swapgs
  mov gs:12, rsp # store user rsp -> scratch at TSS.sp1
  mov rsp, gs:4 # load kernel rsp <- TSS.sp0
  swapgs
  save
  mov rdi, rsp
  call syscall_handler
  mov [rsp], rax # CallerRegs.rax is at offset 0
  jmp __syscall_return

.global syscall_return
syscall_return: # (CallerRegs *)
  mov rsp, rdi
__syscall_return:
  lea rax, [rsp + 72] # prepare new TSS.sp0, 72 = sizeof(CallerRegs)
  swapgs
  mov gs:4, rax
  restore
  mov rsp, gs:12 # restore user rsp
  swapgs
  sysretq
