use crate::WRITER;

pub fn command_echo(args: &str) {
    // כאן אתה משתמש ב-print! של הקרנל שלך
    crate::WRITER.get().unwrap().lock().println(args);
}

pub fn command_help(_args: &str) {
    crate::WRITER
        .get()
        .unwrap()
        .lock()
        .println("Available commands: echo, clear, help");
}

pub fn clear(_args: &str) {
    crate::WRITER.get().unwrap().lock().clear_screen();
}

pub fn command_disktest(_args: &str) {
    let mut buf = [0u8; 512];
    crate::ata::read_sector(0, &mut buf);

    let writer = crate::WRITER.get().unwrap();
    let mut w = writer.lock();

    if buf[510] == 0x55 && buf[511] == 0xAA {
        w.println("Disk OK! FAT16 signature found: 55 AA");
    } else {
        w.println("No valid disk signature");
    }
}

pub fn command_write(args: &str) {
    // פורמט: WRITE filename.txt תוכן
    let mut parts = args.splitn(2, ' ');
    let filename = parts.next().unwrap_or("");
    let content = parts.next().unwrap_or("");

    // חלק את השם לname + ext
    let mut name_ext = filename.splitn(2, '.');
    let name = name_ext.next().unwrap_or("FILE");
    let ext = name_ext.next().unwrap_or("TXT");

    if crate::fat::create_file(name, ext, content.as_bytes()) {
        crate::WRITER.get().unwrap().lock().println("File created!");
    } else {
        crate::WRITER
            .get()
            .unwrap()
            .lock()
            .println("Error: disk full or dir full");
    }
}

pub fn command_read(args: &str) {
    let mut name_ext = args.splitn(2, '.');
    let name = name_ext.next().unwrap_or("");
    let ext = name_ext.next().unwrap_or("");

    let mut buf = [0u8; 512];
    let size = crate::fat::read_file(name, ext, &mut buf);

    if size == 0 {
        crate::WRITER
            .get()
            .unwrap()
            .lock()
            .println("File not found");
        return;
    }

    let content = core::str::from_utf8(&buf[..size as usize]).unwrap_or("???");
    crate::WRITER.get().unwrap().lock().println(content);
}

pub fn command_delete(args: &str) {
    let mut name_ext = args.splitn(2, '.');
    let name = name_ext.next().unwrap_or("");
    let ext = name_ext.next().unwrap_or("");

    let successfully: bool = crate::fat::delete_file(name, ext);
    crate::WRITER
        .get()
        .unwrap()
        .lock()
        .println(if successfully {
            "deleted successfully"
        } else {
            "deleting file failed"
        });
}

pub fn commeand_list(args: &str) {
    let raw_buf = crate::fat::list_files();
    let string_list = core::str::from_utf8(&raw_buf).unwrap_or("");
    crate::WRITER.get().unwrap().lock().println(string_list);
}
