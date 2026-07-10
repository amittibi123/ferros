
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
