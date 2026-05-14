
#!/bin/bash

set -e



cargo build



if [ ! -d "limine" ]; then

    git clone https://github.com/limine-bootloader/limine.git --branch=v8.x-binary --depth=1

    make -C limine

fi



mkdir -p iso/boot

cp target/x86_64-unknown-none/debug/my_os iso/boot/kernel

cp limine/limine-bios.sys limine/limine-bios-cd.bin iso/



printf 'timeout: 0\n\n/My OS\n    protocol: limine\n    kernel_path: boot():/boot/kernel\n' > iso/limine.conf



xorriso -as mkisofs -b limine-bios-cd.bin -no-emul-boot -boot-load-size 4 -boot-info-table --protective-msdos-label iso -o my_os.iso 2>/dev/null



./limine/limine bios-install my_os.iso 2>/dev/null



if [ ! -f "disk.img" ]; then

    dd if=/dev/zero of=disk.img bs=1M count=10

fi



echo "✅ נבנה בהצלחה!"

qemu-system-x86_64 -cdrom my_os.iso -drive file=disk.img,format=raw,id=disk0 -display gtk

