pub static TEST_PROGRAM: [u8; 9] = [
    0xB0, 0x55,             // mov al, 0x55
    0x66, 0xBA, 0xE9, 0x00, // mov dx, 0xE9
    0xEE,                   // out dx, al
    0xEB, 0xF7,             // jmp back to start (-9)
];
