; userspace: קריאה ל-print syscall
.global _start

.section .data
msg: .ascii "Hello, Ferros!\n"
msg_len: .quad 15

.section .text
_start:
    mov rax, 1          ; syscall number: print
    mov rdi, msg        ; pointer to string
    mov rsi, [msg_len]  ; length
    syscall