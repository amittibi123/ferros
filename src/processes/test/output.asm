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

    lea rdi, [user_input]
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
    ; --- print_raw "did not find cmd: " ---
    mov rdx, 18
    xor eax, eax
    lea rsi, [str_inline_2]
    syscall

    ; --- print cmd (with newline) ---
    mov rsi, [cmd]
    xor rdx, rdx
.strlen_3_loop:
    cmp byte [rsi + rdx], 0
    je .strlen_3_done
    inc rdx
    jmp .strlen_3_loop
.strlen_3_done:
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

    ; נשמור את כתובת החזרה (כי אנחנו הולכים להשתמש במחסנית)

    pop r9                  ; r9 מחזיק זמנית את לאן לחזור בסוף הפונקציה



.loop_tokens:

    ; 1. דילוג על רווחים בהתחלה

.skip_spaces:

    mov cl, [rdi]

    cmp cl, 0

    je .done                ; סוף המחרוזת

    cmp cl, ' '

    jne .found_token        ; מצאנו אות, זו תחילת מילה

    inc rdi

    jmp .skip_spaces



.found_token:

    ; 2. דוחפים את הכתובת של תחילת המילה ישירות למחסנית!

    push rdi



    ; 3. מחפשים את סוף המילה הנוכחית

.find_end:

    mov cl, [rdi]

    cmp cl, 0

    je .done                ; סוף המחרוזת

    cmp cl, ' '

    je .null_terminate      ; מצאנו רווח, צריך לחתוך

    inc rdi

    jmp .find_end



.null_terminate:

    mov byte [rdi], 0       ; מחליפים את הרווח ב-NULL (0)

    inc rdi                 ; מתקדמים תו אחד קדימה

    jmp .loop_tokens        ; ממשיכים לחפש את המילה הבאה



.done:

    ; מחזירים את כתובת החזרה למחסנית כדי ש-ret יעבוד כמו שצריך

    push r9

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
str_inline_2: db "did not find cmd: "
