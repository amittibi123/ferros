#!/bin/bash
set -e

# 1. קומפילציה של הקרנל ב-Rust
cargo build

# 2. טיפול ב-Bootloader (Limine)
if [ ! -d "limine" ]; then
    git clone https://github.com/limine-bootloader/limine.git --branch=v8.x-binary --depth=1
    # על מק, אנחנו מבקשים מ-make לבנות אך ורק את כלי ה-host הנייטיבי של למיני
    make -C limine limine
fi

mkdir -p iso/boot

# 3. העתקת הקרנל של Ferros
cp target/x86_64-unknown-none/debug/ferros iso/boot/kernel

# 4. העתקת קבצי ה-BIOS של למיני (הם כבר קיימים שם ב-v8.x-binary)
cp limine/limine-bios.sys limine/limine-bios-cd.bin iso/

# 5. יצירת קובץ הקונפיגורציה
printf 'timeout: 0\n\n/My OS\n    protocol: limine\n    kernel_path: boot():/boot/kernel\n' > iso/limine.conf

# 6. יצירת קובץ ה-ISO (בלי להשתיק שגיאות כדי שנראה אם משהו לא מותקן)
xorriso -as mkisofs -b limine-bios-cd.bin -no-emul-boot -boot-load-size 4 -boot-info-table --protective-msdos-label -partition_offset 16 iso -o my_os.iso

# 7. התקנת ה-Bootloader על קובץ ה-ISO
./limine/limine bios-install my_os.iso

# 8. יצירת דיסק קשיח וירטואלי אם לא קיים
if [ ! -f "disk.img" ]; then
    dd if=/dev/zero of=disk.img bs=1M count=10
fi

echo "✅ Ferros נבנתה בהצלחה!"

# 9. הרצה ב-QEMU עם תצוגת cocoa שמתאימה למק (במקום gtk)
qemu-system-x86_64 -boot d -cdrom my_os.iso -drive file=disk.img,format=raw,id=disk0 -display cocoa -debugcon stdio -global isa-debugcon.iobase=0xE9 -d int,cpu_reset -D qemu.log -no-reboot -no-shutdown
