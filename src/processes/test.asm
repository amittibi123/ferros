BITS 64
ORG 0x500000

start:
    ; --- 1. הדפסת הודעת פתיחה ---
    xor eax, eax            ; syscall number 0 (sys_print)
    lea rsi, [msg]
    mov rdx, 12             ; האורך של "Hello World" + תו 10 = 12
    syscall

    ; --- 2. הדפסת בקשת קלט ---
    xor eax, eax            ; syscall number 0
    lea rsi, [new]
    mov rdx, 14             ; האורך של "type your msg" + תו 10 = 14
    syscall

wait_for_line:
    ; --- 3. קריאה לסיסקול מקלדת ---
    mov eax, 1              ; syscall number 1 (sys_read_line)
    lea rsi, [user_input]
    syscall                 ; מחזיר אורך ב-RAX (הקרנל ימתין פה ל-Enter)

    mov r12, rax            ; שמירת האורך שחזר

    ; --- 4. הדפסת ירידת שורה אסתטית לאחר הקלט ---
    xor eax, eax
    lea rsi, [newline]
    mov rdx, 2              ; מדפיס את 2 ירידות השורה שהגדרנו ב-newline
    syscall

    ; --- 5. הדפסת השורה שהמשתמש הקליד ---
    mov rdx, r12
    xor eax, eax            ; syscall number 0 (sys_print)
    lea rsi, [user_input]
    syscall

    ; --- 6. עוד ירידת שורה אסתטית בסוף ---
    xor eax, eax
    lea rsi, [newline]
    mov rdx, 2
    syscall

    ; חזרה חלילה לקבלת השורה הבאה
    jmp wait_for_line
; --- אזור הנתונים המתוקן ---
align 8
msg:      db "Hello World", 10        ; התו 10 הוא ירידת שורה (\n). אורך אמיתי: 12 בתים
new:      db "type your msg", 10      ; אורך אמיתי: 14 בתים
newline:  db 10, 10                   ; שני תווים של ירידת שורה (כי ביקשת אורך 2 ב-rdx)
user_input: times 64 db 0             ; באפר לקלט
