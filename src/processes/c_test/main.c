#include "ferros.h"

static inline void dispatcher(const FString raw_cmd) {
    const FString cmd = fstring_first_word(raw_cmd);
    const FString args = fstring_after_first_word(raw_cmd);
    if (fstr_eq(cmd, "ECHO")) {
        printf("%c\n", args);
    } else if (fstr_eq(cmd, "HELP")) {
        printf("help clear echo\n");
    }
}

void _start() {
    // הדפסת מחרוזת פשוטה (%c עבור סטרינג C קלאסי)
    printf("%c", "Welcome to Ferros OS!\n");

    char input_buf[100];
    while (1) {
        printf("%c>", "/");
        FString cmd = input(input_buf, 100);

        dispatcher(cmd);
    }
}
