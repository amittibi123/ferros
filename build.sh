#!/bin/bash
set -e

# קמפל
cargo build

# הורד limine אם לא קיים
if [ ! -d "limine" ]; then
    git clone https://github.com/limine-bootloader/limine.git --branch=v8.x-binary --depth=1
    make -C limine
fi

# צור ISO
mkdir -p iso/boot
cp target/x86_64-unknown-none/debug/my_os iso/boot/kernel
cp limine/limine-bios.sys limine/limine-bios-cd.bin iso/

cat > iso/limine.cfg << LIMCFG
timeout: 0

/My OS
    protocol: limine
    kernel_path: boot():/boot/kernel
LIMCFG

xorriso -as mkisofs \
    -b limine-bios-cd.bin \
    -no-emul-boot -boot-load-size 4 -boot-info-table \
    --protective-msdos-label \
    iso -o my_os.iso 2>/dev/null

./limine/limine bios-install my_os.iso 2>/dev/null

echo "✅ נבנה בהצלחה!"
