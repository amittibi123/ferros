import sys

def _emit_if_cmp(var_name, target_val, branch_id, asm_data):
    cmp_label = f"str_cmp_{branch_id}"
    asm_data.append(f"{cmp_label}: db \"{target_val}\", 0")

    lines = [f"    ; --- if {var_name} == '{target_val}' ---"]
    lines.append(f"    mov rsi, [{var_name}]")
    lines.append(f"    lea rdi, [{cmp_label}]")
    lines.append(f".if_cmp_{branch_id}:")
    lines.append(f"    mov al, [rsi]")
    lines.append(f"    mov bl, [rdi]")
    lines.append(f"    cmp al, bl")
    lines.append(f"    jne .if_else_{branch_id}")
    lines.append(f"    cmp al, 0")
    lines.append(f"    je .if_true_{branch_id}")
    lines.append(f"    inc rsi")
    lines.append(f"    inc rdi")
    lines.append(f"    jmp .if_cmp_{branch_id}")
    lines.append(f".if_true_{branch_id}:")
    return lines

def compile_code(input_file, output_file):
    try:
        with open(input_file, 'r', encoding='utf-8') as f:
            lines = f.readlines()
    except FileNotFoundError:
        print(f"Error: קובץ הקלט {input_file} לא נמצא.")
        return

    asm_code = []
    asm_data = []
    asm_bss = []  # <<< רשימה חדשה עבור משתני .bss
    vars = []

    # מונים פנימיים ללייבלים ייחודיים
    inline_str_count = 0
    if_count = 0
    if_stack = []  # מחסנית למעקב אחרי ה-IF-ים הפתוחים

    libs = []

    for line in lines:
        line = line.strip()
        line = line.split('#')[0].strip()

        if not line:
            continue

        parts = line.split(' ', 1)
        command = parts[0]
        args = parts[1].strip() if len(parts) > 1 else ""

        # =================================================================
        # פקודה חדשה: tokens (להגדרת משתנים ריקים ב- .bss)
        # פורמט מצופה: tokens name = size_in_bytes (למשל tokens cmd = 40)
        # או עבור פוינטר: tokens my_ptr = q (מילה מרובעת - 8 בתים)
        # =================================================================
        if command == 'tokens':
            if '=' in args:
                name, size = args.split('=', 1)
                name = name.strip()
                size = size.strip()
                vars.append(name)

                if size == 'q':
                    # הגדרת פוינטר של 64 סיביות (8 בתים) עם יישור זיכרון
                    asm_bss.append("align 8")
                    asm_bss.append(f"{name}: resq 1")
                else:
                    # הגדרת באפר בגודל מותאם אישית בבתים
                    asm_bss.append(f"{name}: resb {size}")
            continue

        # 1. הגדרת משתנה מאותחל (נשאר ב-.data)
        elif command == 'var':
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

        elif command == 'var_q':
            if '=' in args:
                name, val = args.split('=', 1)
                name = name.strip()
                val = val.strip()
                vars.append(name)
                if (val.startswith('"') and val.endswith('"')) or (val.startswith("'") and val.endswith("'")):
                    actual_str = val[1:-1]
                    if actual_str == "\\n" or actual_str == "\n":
                        asm_data.append(f"{name}: dq 10")
                    else:
                        asm_data.append(f"{name}: dq \"{actual_str}\"")
                else:
                    asm_data.append(f"{name}: dq {val}")

        elif command == 'print':
            name = args
            is_input_buffer = False
            is_ptr_var = False
            string_length = 0

            if (name.startswith('"') and name.endswith('"')) or (name.startswith("'") and name.endswith("'")):
                actual_str = name[1:-1]
                string_length = len(actual_str)
                anon_label = f"str_inline_{inline_str_count}"
                inline_str_count += 1
                asm_data.append(f"{anon_label}: db \"{actual_str}\"")
                name = anon_label
            else:
                for bss_line in asm_bss:
                    if bss_line.startswith(f"{name}:"):
                        if "resq" in bss_line:
                            is_ptr_var = True
                        elif "resb" in bss_line:
                            is_input_buffer = True
                        break
                for data_line in asm_data:
                    if data_line.startswith(f"{name}:"):
                        if "times" in data_line:
                            is_input_buffer = True
                        elif '"' in data_line:
                            actual_str = data_line.split('"')[1]
                            string_length = len(actual_str)
                        break

            asm_code.append(f"    ; --- print {args} (with newline) ---")
            if is_ptr_var:
                strlen_label = f"strlen_{inline_str_count}"
                inline_str_count += 1
                asm_code.append(f"    mov rsi, [{name}]")
                asm_code.append(f"    xor rdx, rdx")
                asm_code.append(f".{strlen_label}_loop:")
                asm_code.append(f"    cmp byte [rsi + rdx], 0")
                asm_code.append(f"    je .{strlen_label}_done")
                asm_code.append(f"    inc rdx")
                asm_code.append(f"    jmp .{strlen_label}_loop")
                asm_code.append(f".{strlen_label}_done:")
                asm_code.append(f"    xor eax, eax")
                asm_code.append(f"    syscall")
            elif is_input_buffer:
                asm_code.append(f"    mov rdx, r12")
                asm_code.append(f"    xor eax, eax")
                asm_code.append(f"    lea rsi, [{name}]")
                asm_code.append(f"    syscall")
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
            is_ptr_var = False
            string_length = 0

            if (name.startswith('"') and name.endswith('"')) or (name.startswith("'") and name.endswith("'")):
                actual_str = name[1:-1]
                string_length = len(actual_str)
                anon_label = f"str_inline_{inline_str_count}"
                inline_str_count += 1
                asm_data.append(f"{anon_label}: db \"{actual_str}\"")
                name = anon_label
            else:
                for bss_line in asm_bss:
                    if bss_line.startswith(f"{name}:"):
                        if "resq" in bss_line:
                            is_ptr_var = True
                        elif "resb" in bss_line:
                            is_input_buffer = True
                        break
                for data_line in asm_data:
                    if data_line.startswith(f"{name}:"):
                        if "times" in data_line:
                            is_input_buffer = True
                        elif '"' in data_line:
                            actual_str = data_line.split('"')[1]
                            string_length = len(actual_str)
                        break

            asm_code.append(f"    ; --- print_raw {args} ---")
            if is_ptr_var:
                strlen_label = f"strlen_{inline_str_count}"
                inline_str_count += 1
                asm_code.append(f"    mov rsi, [{name}]")
                asm_code.append(f"    xor rdx, rdx")
                asm_code.append(f".{strlen_label}_loop:")
                asm_code.append(f"    cmp byte [rsi + rdx], 0")
                asm_code.append(f"    je .{strlen_label}_done")
                asm_code.append(f"    inc rdx")
                asm_code.append(f"    jmp .{strlen_label}_loop")
                asm_code.append(f".{strlen_label}_done:")
                asm_code.append(f"    xor eax, eax")
                asm_code.append(f"    syscall\n")
            elif is_input_buffer:
                asm_code.append(f"    mov rdx, r12")
                asm_code.append(f"    xor eax, eax")
                asm_code.append(f"    lea rsi, [{name}]")
                asm_code.append(f"    syscall\n")
            else:
                asm_code.append(f"    mov rdx, {string_length}")
                asm_code.append(f"    xor eax, eax")
                asm_code.append(f"    lea rsi, [{name}]")
                asm_code.append(f"    syscall\n")
        # 4. פקודת קלט: input name (מעביר ל-bss במקום ל-times)
        elif command == 'input':
            name = args
            if name not in vars:
                asm_bss.append(f"{name}: resb 64")
                vars.append(name)

            asm_code.append(f"    ; --- input {name} ---")
            asm_code.append(f"    mov eax, 1")
            asm_code.append(f"    lea rsi, [{name}]")
            asm_code.append(f"    syscall")
            asm_code.append(f"    mov r12, rax\n")

        # 5. הגדרת לייבל/פונקציה
        elif command == 'def':
            asm_code.append(f"{args}:")

        # 6. פקודת קפיצה: goto name
        elif command == 'goto':
            asm_code.append(f"    jmp {args}")

        # 6ב. פקודת קריאה לפונקציה: call name
        elif command == 'call':
                    args = args.strip()
                    if ',' in args:
                        name, arg = args.split(',', 1)
                        name = name.strip()
                        arg = arg.strip()

                        is_input_buffer = False
                        is_ptr_var = False
                        string_length = None

                        if (arg.startswith('"') and arg.endswith('"')) or (arg.startswith("'") and arg.endswith("'")):
                            actual_str = arg[1:-1]
                            string_length = len(actual_str)
                            anon_label = f"str_inline_{inline_str_count}"
                            inline_str_count += 1
                            asm_data.append(f"{anon_label}: db \"{actual_str}\"")
                            arg = anon_label
                        else:
                            for bss_line in asm_bss:
                                if bss_line.startswith(f"{arg}:"):
                                    if "resq" in bss_line:
                                        is_ptr_var = True
                                    elif "resb" in bss_line:
                                        is_input_buffer = True
                                    break
                            for data_line in asm_data:
                                if data_line.startswith(f"{arg}:"):
                                    if "times" in data_line:
                                        is_input_buffer = True
                                    elif '"' in data_line:
                                        actual_str = data_line.split('"')[1]
                                        string_length = len(actual_str)
                                    break

                        asm_code.append(f"    ; --- call {name}, {arg} ---")
                        if is_ptr_var:
                            strlen_label = f"strlen_{inline_str_count}"
                            inline_str_count += 1
                            asm_code.append(f"    mov rdi, [{arg}]")
                            asm_code.append(f"    xor rsi, rsi")
                            asm_code.append(f".{strlen_label}_loop:")
                            asm_code.append(f"    cmp byte [rdi + rsi], 0")
                            asm_code.append(f"    je .{strlen_label}_done")
                            asm_code.append(f"    inc rsi")
                            asm_code.append(f"    jmp .{strlen_label}_loop")
                            asm_code.append(f".{strlen_label}_done:")
                        elif is_input_buffer:
                            asm_code.append(f"    lea rdi, [{arg}]")
                            asm_code.append(f"    mov rsi, r12")
                        elif string_length is not None:
                            asm_code.append(f"    lea rdi, [{arg}]")
                            asm_code.append(f"    mov rsi, {string_length}")
                        else:
                            asm_code.append(f"    lea rdi, [{arg}]")
                            asm_code.append(f"    xor rsi, rsi")

                        asm_code.append(f"    call {name}")
                    else:
                        name = args
                        asm_code.append(f"    call {name}")

        elif command == 'pop':
            var = args
            asm_code.append("    pop rax")
            asm_code.append(f"    mov [{var}], rax ")

        # 6ג. פקודת חזרה מפונקציה: ret
        elif command == 'ret':
            asm_code.append(f"    ret\n")

        elif command == 'if':
            if '==' in args:
                var_name, target_val = args.split('==', 1)
                var_name = var_name.strip()
                target_val = target_val.strip().strip("'").strip('"')

                end_id = if_count
                branch_id = if_count
                if_count += 1
                if_stack.append({'end_id': end_id, 'branch_id': branch_id, 'closed': False})

                asm_code.extend(_emit_if_cmp(var_name, target_val, branch_id, asm_data))
            else:
                raise SyntaxError(f"מבנה IF לא תקין: {line}. חייב להכיל ==")

        elif command == 'elif':
            if not if_stack:
                raise SyntaxError("נמצאה פקודת elif ללא תנאי if פתוח!")
            if '==' not in args:
                raise SyntaxError(f"מבנה ELIF לא תקין: {line}. חייב להכיל ==")

            var_name, target_val = args.split('==', 1)
            var_name = var_name.strip()
            target_val = target_val.strip().strip("'").strip('"')

            top = if_stack[-1]
            asm_code.append(f"    jmp .if_end_{top['end_id']}")
            asm_code.append(f".if_else_{top['branch_id']}:")

            new_branch_id = if_count
            if_count += 1
            top['branch_id'] = new_branch_id

            asm_code.extend(_emit_if_cmp(var_name, target_val, new_branch_id, asm_data))

        # 8. פקודת ELSE
        elif command == 'else':
            if not if_stack:
                raise SyntaxError("נמצאה פקודת else ללא תנאי if פתוח!")
            top = if_stack[-1]
            asm_code.append(f"    jmp .if_end_{top['end_id']}")
            asm_code.append(f".if_else_{top['branch_id']}:")
            top['closed'] = True

        # 9. פקודת ENDIF
        elif command == 'endif':
            if not if_stack:
                raise SyntaxError("נמצאה פקודת endif ללא תנאי if פתוח!")
            top = if_stack.pop()
            if not top['closed']:
                asm_code.append(f".if_else_{top['branch_id']}:")
            asm_code.append(f".if_end_{top['end_id']}:")        # 10. השמה / שינוי תו

        elif command in vars:
            name = command
            if '=' in args:
                _, val = args.split('=', 1)
                val = val.strip().strip("'").strip('"')
                asm_code.append(f"    ; --- set {name} to {val} ---")
                asm_code.append(f"    mov byte [{name}], '{val}'\n")

        # פקודה לטעינת כתובת
        elif command == 'load_ptr':
            target, source = args.split(',', 1)
            target = target.strip()
            source = source.strip()
            asm_code.append(f"    ; --- load_ptr {target}, {source} ---")
            asm_code.append(f"    lea rax, [{source}]")
            asm_code.append(f"    mov [{target}], rax\n")
            if target not in vars:
                asm_bss.append("align 8")
                asm_bss.append(f"{target}: resq 1")
                vars.append(target)

        # פקודה לקריאת תו
        elif command == 'read_char':
            target, ptr = args.split(',', 1)
            target = target.strip()
            ptr = ptr.strip()
            asm_code.append(f"    ; --- read_char {target}, {ptr} ---")
            asm_code.append(f"    mov rbx, [{ptr}]")
            asm_code.append(f"    mov al, [rbx]")
            asm_code.append(f"    mov [{target}], al\n")
            if target not in vars:
                asm_bss.append(f"{target}: resb 1")
                vars.append(target)

        # פקודה לקידום המצביע
        elif command == 'inc_ptr':
            ptr = args.strip()
            asm_code.append(f"    ; --- inc_ptr {ptr} ---")
            asm_code.append(f"    mov rax, [{ptr}]")
            asm_code.append(f"    inc rax")
            asm_code.append(f"    mov [{ptr}], rax\n")

        # פקודה לכתיבת תו
        elif command == 'write_char':
            ptr, char_var = args.split(',', 1)
            ptr = ptr.strip()
            char_var = char_var.strip()
            asm_code.append(f"    ; --- write_char {ptr}, {char_var} ---")
            asm_code.append(f"    mov rbx, [{ptr}]")
            asm_code.append(f"    mov al, [{char_var}]")
            asm_code.append(f"    mov [rbx], al")

        # פקודה להדפסת מחרוזת מתוך מצביע
        elif command == 'print_ptr':
            ptr = args.strip()
            asm_code.append(f"    ; --- print_ptr {ptr} ---")
            asm_code.append(f"    mov rdx, r12")
            asm_code.append(f"    cmp rdx, 0")
            asm_code.append(f"    jle .skip_print_{ptr}")
            asm_code.append(f"    mov rsi, [{ptr}]")
            asm_code.append(f"    xor eax, eax")
            asm_code.append(f"    syscall")
            asm_code.append(f".skip_print_{ptr}:")
            asm_code.append(f"    mov rdx, 1")
            asm_code.append(f"    xor eax, eax")
            asm_code.append(f"    lea rsi, [global_newline]")
            asm_code.append(f"    syscall\n")

        elif command == 'import':
            name = args.strip()
            libs.append(name)

        else:
            raise ValueError(f"Unknown command found: {line}")

    if if_stack:
        raise SyntaxError("שגיאה: שכחת לסגור את אחד מתנאי ה-if באמצעות endif!")

    for f in libs:
        with open(f, 'r', encoding='utf-8') as f:
            lines = f.readlines()
            asm_code.extend(lines)

    # --- בניית הקובץ הסופי ---
    full_output = []
    full_output.append("BITS 64\nORG 0x500000\n")

    # הוספת ה-BSS Section (כדי ש-NASM יידע לשים אותם בנפרד)
    if asm_bss:
        full_output.append("SECTION .bss")
        full_output.extend(asm_bss)
        full_output.append("")

    # חזרה ל-Text/Data הסטנדרטי
    full_output.append("SECTION .text\nglobal start\n\nstart:")
    full_output.extend(asm_code)
    full_output.append("    ; לולאת סיום סתמית")
    full_output.append("inf_loop:\n    jmp inf_loop\n")

    full_output.append("SECTION .data")
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
