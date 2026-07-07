import sys

def compile_code(input_file, output_file):
    try:
        with open(input_file, 'r', encoding='utf-8') as f:
            lines = f.readlines()
    except FileNotFoundError:
        print(f"Error: קובץ הקלט {input_file} לא נמצא.")
        return

    asm_code = []
    asm_data = []
    vars = []

    # מונים פנימיים ללייבלים ייחודיים
    inline_str_count = 0
    if_count = 0
    if_stack = []  # מחסנית למעקב אחרי ה-IF-ים הפתוחים

    for line in lines:
        line = line.strip()
        line = line.split('#')[0].strip()

            # אם השורה הייתה רק קומנט (או שורה ריקה), היא עכשיו ריקה לחלוטין - אז מדלגים
        if not line:
            continue

        parts = line.split(' ', 1)
        command = parts[0]
        args = parts[1].strip() if len(parts) > 1 else ""

        # 1. הגדרת משתנה
        if command == 'var':
            if '=' in args:
                name, val = args.split('=', 1)
                name = name.strip()
                val = val.strip()
                vars.append(name)
                if (val.startswith('"') and val.endswith('"')) or (val.startswith("'") and val.endswith("'")):
                    actual_str = val[1:-1]
                    if actual_str == "\\n" or actual_str == "\n":
                        asm_data.append(f"{name}: db 10")
                    else:
                        asm_data.append(f"{name}: db \"{actual_str}\"")
                else:
                    asm_data.append(f"{name}: db {val}")

        # 2. פקודת הדפסה רגילה (עם שורה חדשה)
        elif command == 'print':
            name = args
            is_input_buffer = False
            string_length = 0

            if (name.startswith('"') and name.endswith('"')) or (name.startswith("'") and name.endswith("'")):
                actual_str = name[1:-1]
                string_length = len(actual_str)
                anon_label = f"str_inline_{inline_str_count}"
                inline_str_count += 1
                asm_data.append(f"{anon_label}: db \"{actual_str}\"")
                name = anon_label
            else:
                for data_line in asm_data:
                    if data_line.startswith(f"{name}:"):
                        if "times" in data_line:
                            is_input_buffer = True
                        elif '"' in data_line:
                            actual_str = data_line.split('"')[1]
                            string_length = len(actual_str)
                        break

            asm_code.append(f"    ; --- print {args} (with newline) ---")
            if is_input_buffer:
                asm_code.append(f"    mov rdx, r12")
            else:
                asm_code.append(f"    mov rdx, {string_length}")

            asm_code.append(f"    xor eax, eax")
            asm_code.append(f"    lea rsi, [{name}]")
            asm_code.append(f"    syscall")
            asm_code.append(f"    mov rdx, 1")
            asm_code.append(f"    xor eax, eax")
            asm_code.append(f"    lea rsi, [global_newline]")
            asm_code.append(f"    syscall\n")

        # 3. פקודת הדפסה ישירה (בלי שורה חדשה)
        elif command == 'print_raw':
            name = args
            is_input_buffer = False
            string_length = 0

            if (name.startswith('"') and name.endswith('"')) or (name.startswith("'") and name.endswith("'")):
                actual_str = name[1:-1]
                string_length = len(actual_str)
                anon_label = f"str_inline_{inline_str_count}"
                inline_str_count += 1
                asm_data.append(f"{anon_label}: db \"{actual_str}\"")
                name = anon_label
            else:
                for data_line in asm_data:
                    if data_line.startswith(f"{name}:"):
                        if "times" in data_line:
                            is_input_buffer = True
                        elif '"' in data_line:
                            actual_str = data_line.split('"')[1]
                            string_length = len(actual_str)
                        break

            asm_code.append(f"    ; --- print_raw {args} ---")
            if is_input_buffer:
                asm_code.append(f"    mov rdx, r12")
            else:
                asm_code.append(f"    mov rdx, {string_length}")

            asm_code.append(f"    xor eax, eax")
            asm_code.append(f"    lea rsi, [{name}]")
            asm_code.append(f"    syscall\n")

        # 4. פקודת קלט: input name
        elif command == 'input':
            name = args
            if name not in vars:
                asm_data.append(f"{name}: times 64 db 0")
                vars.append(name)

            asm_code.append(f"    ; --- input {name} ---")
            asm_code.append(f"    mov eax, 1")
            asm_code.append(f"    lea rsi, [{name}]")
            asm_code.append(f"    syscall")
            asm_code.append(f"    mov r12, rax\n")

        # 5. הגדרת לייבל/פונקציה: גלובלי (בלי נקודה) כדי למנוע התנגשויות
        elif command == 'def':
            asm_code.append(f"{args}:")

        # 6. פקודת קפיצה: goto name
        elif command == 'goto':
            asm_code.append(f"    jmp {args}")

        # 6ב. פקודת קריאה לפונקציה: call name
        elif command == 'call':
            asm_code.append(f"    call {args}")

        # 6ג. פקודת חזרה מפונקציה: ret
        elif command == 'ret':
            asm_code.append(f"    ret\n")

        # 7. פקודת IF
        elif command == 'if':
            if '==' in args:
                var_name, target_val = args.split('==', 1)
                var_name = var_name.strip()
                target_val = target_val.strip().strip("'").strip('"')

                current_if_id = if_count
                if_count += 1
                if_stack.append(current_if_id)

                asm_code.append(f"    ; --- if {var_name} == '{target_val}' ---")
                asm_code.append(f"    mov al, [{var_name}]")
                asm_code.append(f"    cmp al, '{target_val}'")
                asm_code.append(f"    jne .if_else_{current_if_id}") # לייבלים פנימיים של IF נשארים מקומיים
            else:
                raise SyntaxError(f"מבנה IF לא תקין: {line}. חייב להכיל ==")

        # 8. פקודת ELSE
        elif command == 'else':
            if not if_stack:
                raise SyntaxError("נמצאה פקודת else ללא תנאי if פתוח!")
            current_if_id = if_stack[-1]
            asm_code.append(f"    jmp .if_end_{current_if_id}")
            asm_code.append(f".if_else_{current_if_id}:")

        # 9. פקודת ENDIF
        elif command == 'endif':
            if not if_stack:
                raise SyntaxError("נמצאה פקודת endif ללא תנאי if פתוח!")
            current_if_id = if_stack.pop()
            else_label = f".if_else_{current_if_id}:"
            if not any(else_label in code_line for code_line in asm_code):
                asm_code.append(else_label)
            asm_code.append(f".if_end_{current_if_id}:")

        # 10. השמה / שינוי תו
        elif command in vars:
            name = command
            if '=' in args:
                _, val = args.split('=', 1)
                val = val.strip().strip("'").strip('"')
                asm_code.append(f"    ; --- set {name} to {val} ---")
                asm_code.append(f"    mov byte [{name}], '{val}'\n")

# פקודה לטעינת כתובת של באפר לתוך משתנה מצביע: load_ptr my_ptr, char_input
        elif command == 'load_ptr':
            target, source = args.split(',', 1)
            target = target.strip()
            source = source.strip()
            asm_code.append(f"    ; --- load_ptr {target}, {source} ---")
            asm_code.append(f"    lea rax, [{source}]")
            asm_code.append(f"    mov [{target}], rax\n")
            if target not in vars:
                asm_data.append(f"{target}: dq 0") # dq בשביל לשמור כתובת של 64-ביט
                vars.append(target)

        # פקודה לקריאת התו הנוכחי שהמצביע מראה עליו לתוך משתנה: read_char my_char, my_ptr
        elif command == 'read_char':
            target, ptr = args.split(',', 1)
            target = target.strip()
            ptr = ptr.strip()
            asm_code.append(f"    ; --- read_char {target}, {ptr} ---")
            asm_code.append(f"    mov rbx, [{ptr}]")
            asm_code.append(f"    mov al, [rbx]")
            asm_code.append(f"    mov [{target}], al\n")
            if target not in vars:
                asm_data.append(f"{target}: db 0")
                vars.append(target)

        # פקודה לקידום המצביע לתו הבא: inc_ptr my_ptr
        elif command == 'inc_ptr':
            ptr = args.strip()
            asm_code.append(f"    ; --- inc_ptr {ptr} ---")
            asm_code.append(f"    mov rax, [{ptr}]")
            asm_code.append(f"    inc rax")
            asm_code.append(f"    mov [{ptr}], rax\n")
# פקודה לכתיבת תו מתוך משתנה אל הכתובת שהמצביע מראה עליה: write_char ptr_var, char_var
        elif command == 'write_char':
            ptr, char_var = args.split(',', 1)
            ptr = ptr.strip()
            char_var = char_var.strip()
            asm_code.append(f"    ; --- write_char {ptr}, {char_var} ---")
            asm_code.append(f"    mov rbx, [{ptr}]")         # טוען את הכתובת שבמצביע ל-RBX
            asm_code.append(f"    mov al, [{char_var}]")      # טוען את התו מתוך המשתנה ל-AL
            asm_code.append(f"    mov [rbx], al")             # כותב את התו לכתובת שב-RBX\n")
# פקודה להדפסת מחרוזת מתוך הכתובת שבמצביע: print_ptr my_ptr
        elif command == 'print_ptr':
                    ptr = args.strip()
                    asm_code.append(f"    ; --- print_ptr {ptr} ---")
                    asm_code.append(f"    mov rdx, r12")       # אורך המחרוזת
                    asm_code.append(f"    cmp rdx, 0")         # בדיקה: האם האורך הוא 0?
                    asm_code.append(f"    jle .skip_print_{ptr}") # אם הוא 0 או שלילי - אל תדפיס כלום כדי למנוע קריסה
                    asm_code.append(f"    mov rsi, [{ptr}]")   # טעינת הכתובת הדינמית
                    asm_code.append(f"    xor eax, eax")       # sys_print
                    asm_code.append(f"    syscall")
                    asm_code.append(f".skip_print_{ptr}:")
                    # ירידת שורה קבועה
                    asm_code.append(f"    mov rdx, 1")
                    asm_code.append(f"    xor eax, eax")
                    asm_code.append(f"    lea rsi, [global_newline]")
                    asm_code.append(f"    syscall\n")

        else:
            raise ValueError(f"Unknown command found: {command}")

    if if_stack:
        raise SyntaxError("שגיאה: שכחת לסגור את אחד מתנאי ה-if באמצעות endif!")

    full_output = []
    full_output.append("BITS 64\nORG 0x500000\n\nstart:")
    full_output.extend(asm_code)
    full_output.append("    ; לולאת סיום סתמית")
    full_output.append("inf_loop:\n    jmp inf_loop\n")
    full_output.append("align 8")
    full_output.append("global_newline: db 10")
    full_output.extend(asm_data)

    with open(output_file, 'w', encoding='utf-8') as f:
        f.write("\n".join(full_output) + "\n")

    print(f"הקומפילציה הסתיימה! הקוד נשמר ב-{output_file}")

if __name__ == "__main__":
    if len(sys.argv) < 3:
        compile_code("input.txt", "output.asm")
    else:
        compile_code(sys.argv[1], sys.argv[2])