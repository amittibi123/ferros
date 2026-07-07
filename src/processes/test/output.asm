BITS 64
ORG 0x500000

start:
    jmp main
print_dir:
    ; --- print_raw dir ---
    mov rdx, 1
    xor eax, eax
    lea rsi, [dir]
    syscall

    ; --- print_raw ">" ---
    mov rdx, 1
    xor eax, eax
    lea rsi, [str_inline_0]
    syscall

    ret

main:
    ; --- print "Welcome to Ferros Shell!" (with newline) ---
    mov rdx, 24
    xor eax, eax
    lea rsi, [str_inline_1]
    syscall
    mov rdx, 1
    xor eax, eax
    lea rsi, [global_newline]
    syscall

shell_loop:
    call print_dir
    ; --- input cmd ---
    mov eax, 1
    lea rsi, [cmd]
    syscall
    mov r12, rax

    call parse_cmd
    call dispatcher
    jmp shell_loop
    ret

parse_cmd:
    ; --- load_ptr my_ptr, cmd ---
    lea rax, [cmd]
    mov [my_ptr], rax

    ; --- load_ptr dest_ptr, cmd_part ---
    lea rax, [cmd_part]
    mov [dest_ptr], rax

read_command_loop:
    ; --- read_char current_char, my_ptr ---
    mov rbx, [my_ptr]
    mov al, [rbx]
    mov [current_char], al

    ; --- if current_char == ' ' ---
    mov al, [current_char]
    cmp al, ' '
    jne .if_else_0
    jmp end_command
.if_else_0:
.if_end_0:
    ; --- if current_char == '0' ---
    mov al, [current_char]
    cmp al, '0'
    jne .if_else_1
    jmp end_command
.if_else_1:
.if_end_1:
    ; --- if current_char == '10' ---
    mov al, [current_char]
    cmp al, '10'
    jne .if_else_2
    jmp end_command
.if_else_2:
.if_end_2:
    ; --- write_char dest_ptr, current_char ---
    mov rbx, [dest_ptr]
    mov al, [current_char]
    mov [rbx], al
    ; --- inc_ptr my_ptr ---
    mov rax, [my_ptr]
    inc rax
    mov [my_ptr], rax

    ; --- inc_ptr dest_ptr ---
    mov rax, [dest_ptr]
    inc rax
    mov [dest_ptr], rax

    jmp read_command_loop
end_command:
    ; --- inc_ptr my_ptr ---
    mov rax, [my_ptr]
    inc rax
    mov [my_ptr], rax

    ret

dispatcher:
    ; --- if cmd_part == 'E' ---
    mov al, [cmd_part]
    cmp al, 'E'
    jne .if_else_3
    call do_echo
    jmp .if_end_3
.if_else_3:
    ; --- print "Unknown command." (with newline) ---
    mov rdx, 16
    xor eax, eax
    lea rsi, [str_inline_2]
    syscall
    mov rdx, 1
    xor eax, eax
    lea rsi, [global_newline]
    syscall

.if_end_3:
    ret

do_echo:
    ; --- print_ptr my_ptr ---
    mov rdx, r12
    cmp rdx, 0
    jle .skip_print_my_ptr
    mov rsi, [my_ptr]
    xor eax, eax
    syscall
.skip_print_my_ptr:
    mov rdx, 1
    xor eax, eax
    lea rsi, [global_newline]
    syscall

    ret

print_args_dynamic:
    ret

    ; לולאת סיום סתמית
inf_loop:
    jmp inf_loop

align 8
global_newline: db 10
dir: db "/"
cmd: db "                                        "
cmd_part: db "          "
current_char: db " "
my_ptr: db 0
dest_ptr: db 0
str_inline_0: db ">"
str_inline_1: db "Welcome to Ferros Shell!"
str_inline_2: db "Unknown command."
