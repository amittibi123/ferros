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