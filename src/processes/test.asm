BITS 64
ORG 0x500000

start:
.shell_loop:
    ; --- 1. הדפסת הפרומפט "/> " ---
    xor eax, eax            ; syscall 0 (sys_print)
    lea rsi, [prompt]
    mov rdx, 3              ; אורך "/> " הוא 3 בתים
    syscall

    ; --- 2. קריאה לסיסקול מקלדת לקבלת פקודה ---
    mov eax, 1              ; syscall 1 (sys_read_line)
    lea rsi, [user_input]
    syscall                 ; הקרנל ממתין ל-Enter ומחזיר אורך ב-RAX

    mov r12, rax            ; שמירת האורך הכולל של הקלט ב-r12

    ; אם המשתמש פשוט לחץ Enter בלי לכתוב כלום (אורך 0 או 1 תלוי אם ה-\n נספר)
    cmp r12, 1
    jle .shell_loop

    ; --- 3. בדיקה האם הפקודה מתחילה ב-"echo " ---
    ; פקודת echo חייבת להכיל לפחות 5 תווים ("echo ")
    cmp r12, 5
    jl .unknown_command

    ; הכנת רגיסטרים להשוואת מחרוזות (String Comparison)
    lea rsi, [user_input]
    lea rdi, [echo_cmd]
    mov rcx, 5              ; נשווה בדיוק 5 בתים
    cld                     ; סריקה קדימה
    repe cmpsb              ; משווה בית-בית כל עוד הם שווים
    jne .unknown_command    ; אם יש חוסר התאמה, זו לא פקודת echo

    ; --- 4. ביצוע פקודת echo ---
    ; אם הגענו לכאן, rsi כבר מקודם אוטומטית ב-5 בתים על ידי cmpsb!
    ; הוא מצביע בדיוק על התו הראשון שאחרי ה-"echo " בתוך user_input.
    ; נחשב את אורך המחרוזת שנשארה להדפסה: אורך כולל פחות 5.
    mov rdx, r12
    sub rdx, 5

    ; אם אין טקסט אחרי ה-echo, פשוט נדפיס שורה חדשה
    cmp rdx, 0
    jle .print_newline_only

    xor eax, eax            ; syscall 0 (sys_print)
    ; rsi כבר מוכן ומצביע לטקסט שאחרי ה-echo
    syscall

.print_newline_only:
    xor eax, eax            ; syscall 0 (sys_print)
    lea rsi, [newline]
    mov rdx, 1
    syscall
    jmp .shell_loop

    ; --- 5. טיפול בפקודה לא מוכרת ---
.unknown_command:
    xor eax, eax            ; syscall 0 (sys_print)
    lea rsi, [err_msg]
    mov rdx, 23             ; אורך הודעת השגיאה כולל ה-\n
    syscall
    jmp .shell_loop

align 8
prompt:    db "/> ", 0
echo_cmd:  db "ECHO ", 0
help_cmd: db "help ", 0
err_msg:   db "Command not recognized", 10
newline:   db 10
user_input: times 64 db 0   ; באפר לקלט של עד 64 תווים
