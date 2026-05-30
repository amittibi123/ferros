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

pub fn command_write(args: &str, dir: &mut heapless::String<64>) {
    // פורמט: WRITE filename.txt תוכן
    let mut parts = args.splitn(2, ' ');
    let filename = parts.next().unwrap_or("");
    let content = parts.next().unwrap_or("");

    // חלק את השם לname + ext
    let mut name_ext = filename.splitn(2, '.');
    let name = name_ext.next().unwrap_or("FILE");
    let ext = name_ext.next().unwrap_or("TXT");

    if crate::fat::create_file_at(dir, name, ext, content.as_bytes()) {
        crate::WRITER.get().unwrap().lock().println("File created!");
    } else {
        crate::WRITER
            .get()
            .unwrap()
            .lock()
            .println("Error: disk full or dir full");
    }
}

pub fn command_read(args: &str, dir: &mut heapless::String<64>) {
    let mut name_ext = args.splitn(2, '.');
    let name = name_ext.next().unwrap_or("");
    let ext = name_ext.next().unwrap_or("");

    let mut buf = [0u8; 512];
    let size = crate::fat::read_file_at(dir, name, ext, &mut buf);

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

pub fn command_delete(args: &str, dir: &mut heapless::String<64>) {
    let mut name_ext = args.splitn(2, '.');
    let name = name_ext.next().unwrap_or("");
    let ext = name_ext.next().unwrap_or("");

    let successfully: bool = crate::fat::delete_file_at(dir, name, ext);
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

pub fn commeand_list(args: &str, dir: &mut heapless::String<64>) {
    let raw_buf = crate::fat::list_dir(dir.as_str());
    let string_list = core::str::from_utf8(&raw_buf).unwrap_or("");
    crate::WRITER.get().unwrap().lock().println(string_list);
}

pub fn mkdir(args: &str, dir: &mut heapless::String<64>) {
    let mut parts = args.splitn(2, ' ');
    let name = parts.next().unwrap_or("");
    let successfully: bool = crate::fat::create_dir_at(dir, name);
    crate::WRITER
        .get()
        .unwrap()
        .lock()
        .println(if successfully {
            "crated successfully"
        } else {
            "failed"
        });
}

pub fn rmdir(args: &str, dir: &mut heapless::String<64>) {
    let mut parts = args.splitn(2, ' ');
    let name = parts.next().unwrap_or("");
    let sccessfully: bool = crate::fat::delete_dir_at(dir, name);
    crate::WRITER.get().unwrap().lock().println(if sccessfully {
        "deleted successfully"
    } else {
        "deleting failed"
    });
}

// הפונקציה עכשיו מקבלת רפרנס למחרוזת של heapless ויכולה לעדכן אותה בבטחה
pub fn cd(args: &str, dir: &mut heapless::String<64>) {
    let mut parts = args.splitn(2, ' ');
    let new_dir = parts.next().unwrap_or("");
    let raw_buf = crate::fat::list_dir(dir.as_str());
    let dir_str = core::str::from_utf8(&raw_buf)
        .unwrap_or("")
        .trim_end_matches('\0');
    if !dir_str.contains(new_dir) && new_dir != ".." && new_dir != "/" {
        crate::WRITER
            .get()
            .unwrap()
            .lock()
            .println("dir doesn't exists");
        return;
    }
    match new_dir {
        ".." => pop_directory(dir),
        "/" => {
            dir.clear();
            let _ = dir.push_str("/");
        }
        _ => {
            if dir != "/" {
                let __ = dir.push_str("/");
            }
            let _ = dir.push_str(new_dir);
        }
    }
    // מתעלמים משגיאת גלישה (Overflow) אם הנתיב ארוך מדי, או מטפלים בה
}

pub fn pop_directory(directory: &mut heapless::String<64>) {
    // 1. מוצאים את האינדקס של ה-/ האחרון במחרוזת
    if let Some(last_slash_idx) = directory.rfind('/') {
        // מקרה קצה: אם ה-/ האחרון הוא הלוכסן הראשון והיחיד (ה-Root כמו "/")
        if last_slash_idx == 0 {
            // משאירים רק את ה-/, כלומר חותכים את הכל החל מאינדקס 1
            directory.truncate(1);
        } else {
            // חותכים את המחרוזת בדיוק במיקום של ה-/ האחרון
            // זה ימחק את ה-/ האחרון ואת כל מה שבא אחריו
            directory.truncate(last_slash_idx);
        }
    }
}
