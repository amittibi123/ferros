#include <stdint.h>

static inline int64_t print(const char *msg, uint64_t len) {
    int64_t ret;
    asm volatile (
        "syscall"
        : "=a"(ret)
        : "a"(0), "S"(msg), "d"(len)
        : "rcx", "r11", "memory"
    );
    return ret;
}

void _start() {
    const char *msg = "Hello, Ferros!\n";
    print(msg, 15);

    while (1) {
        asm volatile ("hlt");
    }
}