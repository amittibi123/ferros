BITS 64
ORG 0x500000

SECTION .bss
align 8
cmd: resq 1
align 8
args: resq 1
user_input: resb 64

SECTION .text
global start

start:
    jmp main
printDir:
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
    call printDir
    ; --- input user_input ---
    mov eax, 1
    lea rsi, [user_input]
    syscall
    mov r12, rax

    ; --- call tokenize_to_stack, user_input ---
    lea rdi, [user_input]
    mov rsi, r12
    call tokenize_to_stack
    pop rax
    mov [args], rax 
    pop rax
    mov [cmd], rax 
    call dispatcher
    jmp main
dispatcher:
    ; --- if cmd == 'ECHO' ---
    mov rsi, [cmd]
    lea rdi, [str_cmp_0]
.if_cmp_0:
    mov al, [rsi]
    mov bl, [rdi]
    cmp al, bl
    jne .if_else_0
    cmp al, 0
    je .if_true_0
    inc rsi
    inc rdi
    jmp .if_cmp_0
.if_true_0:
    ; --- print args (with newline) ---
    mov rsi, [args]
    xor rdx, rdx
.strlen_1_loop:
    cmp byte [rsi + rdx], 0
    je .strlen_1_done
    inc rdx
    jmp .strlen_1_loop
.strlen_1_done:
    xor eax, eax
    syscall
    mov rdx, 1
    xor eax, eax
    lea rsi, [global_newline]
    syscall

    jmp .if_end_0
.if_else_0:
    ; --- if cmd == 'HELP' ---
    mov rsi, [cmd]
    lea rdi, [str_cmp_1]
.if_cmp_1:
    mov al, [rsi]
    mov bl, [rdi]
    cmp al, bl
    jne .if_else_1
    cmp al, 0
    je .if_true_1
    inc rsi
    inc rdi
    jmp .if_cmp_1
.if_true_1:
    ; --- print "usage: ..." (with newline) ---
    mov rdx, 10
    xor eax, eax
    lea rsi, [str_inline_2]
    syscall
    mov rdx, 1
    xor eax, eax
    lea rsi, [global_newline]
    syscall

    jmp .if_end_0
.if_else_1:
    ; --- if cmd == 'CLEAR' ---
    mov rsi, [cmd]
    lea rdi, [str_cmp_2]
.if_cmp_2:
    mov al, [rsi]
    mov bl, [rdi]
    cmp al, bl
    jne .if_else_2
    cmp al, 0
    je .if_true_2
    inc rsi
    inc rdi
    jmp .if_cmp_2
.if_true_2:
    call clear_screen
    jmp .if_end_0
.if_else_2:
    ; --- if cmd == 'DISKTEST' ---
    mov rsi, [cmd]
    lea rdi, [str_cmp_3]
.if_cmp_3:
    mov al, [rsi]
    mov bl, [rdi]
    cmp al, bl
    jne .if_else_3
    cmp al, 0
    je .if_true_3
    inc rsi
    inc rdi
    jmp .if_cmp_3
.if_true_3:
    ; --- call disk_test, dir ---
    lea rdi, [dir]
    mov rsi, 1
    call disk_test
    jmp .if_end_0
.if_else_3:
    ; --- if cmd == 'WRITE' ---
    mov rsi, [cmd]
    lea rdi, [str_cmp_4]
.if_cmp_4:
    mov al, [rsi]
    mov bl, [rdi]
    cmp al, bl
    jne .if_else_4
    cmp al, 0
    je .if_true_4
    inc rsi
    inc rdi
    jmp .if_cmp_4
.if_true_4:
    ; --- call write_to_disk, args ---
    mov rdi, [args]
    xor rsi, rsi
.strlen_3_loop:
    cmp byte [rdi + rsi], 0
    je .strlen_3_done
    inc rsi
    jmp .strlen_3_loop
.strlen_3_done:
    call write_to_disk
    jmp .if_end_0
.if_else_4:
    ; --- if cmd == 'LS' ---
    mov rsi, [cmd]
    lea rdi, [str_cmp_5]
.if_cmp_5:
    mov al, [rsi]
    mov bl, [rdi]
    cmp al, bl
    jne .if_else_5
    cmp al, 0
    je .if_true_5
    inc rsi
    inc rdi
    jmp .if_cmp_5
.if_true_5:
    call ls_disk
    jmp .if_end_0
.if_else_5:
    ; --- print_raw "did not find cmd: " ---
    mov rdx, 18
    xor eax, eax
    lea rsi, [str_inline_4]
    syscall

    ; --- print cmd (with newline) ---
    mov rsi, [cmd]
    xor rdx, rdx
.strlen_5_loop:
    cmp byte [rsi + rdx], 0
    je .strlen_5_done
    inc rdx
    jmp .strlen_5_loop
.strlen_5_done:
    xor eax, eax
    syscall
    mov rdx, 1
    xor eax, eax
    lea rsi, [global_newline]
    syscall

.if_end_0:
    ret

; =====================================================================

; פונקציה: tokenize_to_stack

; קלט:   RDI = כתובת תחילת המחרוזת

; פלט:   דוחפת את הכתובות של המילים ישירות ל-Stack

; =====================================================================

tokenize_to_stack:

    pop r9                  ; כתובת חזרה

    xor r10, r10            ; מונה טוקנים שנדחפו

.loop_tokens:

.skip_spaces:

    mov cl, [rdi]

    cmp cl, 0

    je .check_pad

    cmp cl, ' '

    jne .found_token

    inc rdi

    jmp .skip_spaces

.found_token:

    push rdi

    inc r10

.find_end:

    mov cl, [rdi]

    cmp cl, 0

    je .check_pad

    cmp cl, ' '

    je .null_terminate

    inc rdi

    jmp .find_end

.null_terminate:

    mov byte [rdi], 0

    inc rdi

    jmp .loop_tokens

.check_pad:

    cmp r10, 2

    jge .done

    lea rax, [empty_string]

    push rax

    inc r10

    jmp .check_pad

.done:

    push r9

    ret



empty_string: db 0


clear_screen:

    mov eax, 2

    syscall

    ret



disk_test:

    mov eax, 10

    syscall

    ret



write_to_disk:

    mov eax, 11

    mov rdx, rsi      ; קודם שומרים rdx = args_len

    mov rsi, rdi      ; ורק אז rsi = args_ptr

    call decode_dir

    syscall

    ret



ls_disk:

    mov eax, 12

    xor rsi, rsi

    xor rdx, rdx

    call decode_dir

    syscall

    ret



decode_dir:

        lea r8, [dir]

        xor r9, r9

        .strlen_dir_loop:

            cmp byte [r8 + r9], 0

            je .strlen_dir_done

            inc r9

            jmp .strlen_dir_loop

        .strlen_dir_done:

            mov r10, r8      ; רק עכשיו rdi מקבל את הכתובת

            mov r8, r9      ; ורק עכשיו rsi מקבל את האורך



        ret

    ; לולאת סיום סתמית
inf_loop:
    jmp inf_loop

SECTION .data
align 8
global_newline: db 10
dir: db "/"
str_inline_0: db ">"
str_cmp_0: db "ECHO", 0
str_cmp_1: db "HELP", 0
str_inline_2: db "usage: ..."
str_cmp_2: db "CLEAR", 0
str_cmp_3: db "DISKTEST", 0
str_cmp_4: db "WRITE", 0
str_cmp_5: db "LS", 0
str_inline_4: db "did not find cmd: "
