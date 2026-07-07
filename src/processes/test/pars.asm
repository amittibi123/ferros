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
