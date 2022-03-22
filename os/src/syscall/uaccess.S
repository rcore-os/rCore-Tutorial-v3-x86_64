.text

.global copy_user_start
copy_user_start:

.global copy_user8
copy_user8:
  mov al, byte ptr [rsi]
  mov byte ptr [rdi], al
  xor eax, eax
  ret

.global copy_user16
copy_user16:
  movzx eax, word ptr [rsi]
  mov word ptr [rdi], ax
  xor eax, eax
  ret

.global copy_user32
copy_user32:
  mov eax, dword ptr [rsi]
  mov dword ptr [rdi], eax
  xor eax, eax
  ret

.global copy_user64
copy_user64:
  mov rax, qword ptr [rsi]
  mov qword ptr [rdi], rax
  xor eax, eax
  ret

.global copy_user_n
copy_user_n:
loop:
  sub rdx, 1
  movzx eax, byte ptr [rsi + rdx]
  mov byte ptr [rdi + rdx], al
  jne loop
  xor eax, eax
  ret

.global copy_user_end
copy_user_end:

.global copy_user_fail
copy_user_fail:
  mov eax, 1
  ret
