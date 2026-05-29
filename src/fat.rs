use crate::ata;

// קרא את המידע מה-Boot Record והחזר את הכתובות החשובות
fn get_layout() -> (u32, u32, u32, u32) {
    let mut boot = [0u8; 512];
    ata::read_sector(0, &mut boot);

    let reserved = u16::from_le_bytes([boot[14], boot[15]]) as u32;
    let fat_size = u16::from_le_bytes([boot[22], boot[23]]) as u32;
    let root_count = u16::from_le_bytes([boot[17], boot[18]]) as u32;
    let spc = boot[13] as u32; // sectors per cluster

    let fat_start = reserved;
    let root_start = fat_start + 2 * fat_size;
    let data_start = root_start + (root_count * 32 / 512);

    (fat_start, root_start, data_start, spc)
}

// מצא cluster פנוי ב-FAT Table
fn find_free_cluster(fat_start: u32) -> Option<u16> {
    let mut buf = [0u8; 512];
    ata::read_sector(fat_start, &mut buf);

    for i in 2..256usize {
        let entry = u16::from_le_bytes([buf[i * 2], buf[i * 2 + 1]]);
        if entry == 0x0000 {
            return Some(i as u16);
        }
    }
    None
}

// סמן cluster כ"סוף קובץ" ב-FAT Table
fn mark_cluster_used(fat_start: u32, cluster: u16) {
    let mut buf = [0u8; 512];
    ata::read_sector(fat_start, &mut buf);
    buf[cluster as usize * 2] = 0xFF;
    buf[cluster as usize * 2 + 1] = 0xFF;
    ata::write_sector(fat_start, &buf);
}

// כתוב קובץ חדש לדיסק
// name: שם עד 8 תווים, ext: סיומת עד 3 תווים, content: תוכן עד 512 בייטים
pub fn create_file(name: &str, ext: &str, content: &[u8]) -> bool {
    let (fat_start, root_start, data_start, spc) = get_layout();

    // מצא cluster פנוי
    let cluster = match find_free_cluster(fat_start) {
        Some(c) => c,
        None => return false, // דיסק מלא
    };

    // כתוב את התוכן ל-Data
    let mut data = [0u8; 512];
    let len = content.len().min(512);
    data[..len].copy_from_slice(&content[..len]);
    let sector = data_start + (cluster as u32 - 2) * spc;
    ata::write_sector(sector, &data);

    // סמן את ה-cluster כתפוס ב-FAT
    mark_cluster_used(fat_start, cluster);

    // הוסף entry ב-Root Directory
    let mut root = [0u8; 512];
    ata::read_sector(root_start, &mut root);

    // מצא entry פנוי (מתחיל ב-0x00)
    for i in 0..16usize {
        if root[i * 32] == 0x00 {
            let mut entry = [0x20u8; 32]; // 0x20 = רווח

            // שם קובץ - 8 תווים
            let name_bytes = name.as_bytes();
            let name_len = name_bytes.len().min(8);
            entry[..name_len].copy_from_slice(&name_bytes[..name_len]);

            // סיומת - 3 תווים
            let ext_bytes = ext.as_bytes();
            let ext_len = ext_bytes.len().min(3);
            entry[8..8 + ext_len].copy_from_slice(&ext_bytes[..ext_len]);

            // cluster התחלה
            entry[26] = cluster as u8;
            entry[27] = (cluster >> 8) as u8;

            // גודל קובץ
            entry[28] = len as u8;
            entry[29] = (len >> 8) as u8;
            entry[30] = 0;
            entry[31] = 0;

            root[i * 32..(i + 1) * 32].copy_from_slice(&entry);
            ata::write_sector(root_start, &mut root);
            return true;
        }
    }

    false // root directory מלא
}

pub fn read_file(name: &str, ext: &str, buf: &mut [u8; 512]) -> u32 {
    let (_, root_start, data_start, spc) = get_layout();

    let mut root = [0u8; 512];
    ata::read_sector(root_start, &mut root);

    for i in 0..16 {
        let entry = &root[i * 32..(i + 1) * 32];
        if entry[0] == 0x00 {
            break;
        }
        if entry[0] == 0xE5 {
            continue;
        }

        let disk_name = core::str::from_utf8(&entry[0..8]).unwrap_or("").trim_end();
        let disk_ext = core::str::from_utf8(&entry[8..11]).unwrap_or("").trim_end();

        if disk_name.eq_ignore_ascii_case(name) && disk_ext.eq_ignore_ascii_case(ext) {
            let cluster = u16::from_le_bytes([entry[26], entry[27]]);
            let size = u32::from_le_bytes([entry[28], entry[29], entry[30], entry[31]]);
            let sector = data_start + (cluster as u32 - 2) * spc;
            ata::read_sector(sector, buf);
            return size;
        }
    }
    0
}

pub fn delete_file(name: &str, ext: &str) -> bool {
    let (_, root_start, _, _) = get_layout();

    let mut root = [0u8; 512];
    ata::read_sector(root_start, &mut root);

    for i in 0..16 {
        let entry = &mut root[i * 32..(i + 1) * 32];
        if entry[0] == 0x00 {
            break;
        }
        if entry[0] == 0xE5 {
            continue;
        }

        let disk_name = core::str::from_utf8(&entry[0..8]).unwrap_or("").trim_end();
        let disk_ext = core::str::from_utf8(&entry[8..11]).unwrap_or("").trim_end();

        if disk_name.eq_ignore_ascii_case(name) && disk_ext.eq_ignore_ascii_case(ext) {
            entry[0] = 0xE5;
            ata::write_sector(root_start, &root);
            return true;
        }
    }
    false
}

pub fn list_files() -> [u8; 64] {
    let (_, root_start, _, _) = get_layout();

    let mut root = [0u8; 512];
    ata::read_sector(root_start, &mut root);

    let mut buf = [0u8; 64];
    let mut len = 0;

    for i in 0..16 {
        let entry = &root[i * 32..(i + 1) * 32];

        if entry[0] == 0x00 {
            break;
        }
        if entry[0] == 0xE5 {
            continue;
        }

        let disk_name = core::str::from_utf8(&entry[0..8]).unwrap_or("").trim();
        let disk_ext = core::str::from_utf8(&entry[8..11]).unwrap_or("").trim();

        // העתקת שם הקובץ
        for &b in disk_name.as_bytes() {
            if len < buf.len() {
                buf[len] = b;
                len += 1;
            }
        }

        // הוספת נקודה לפני הסיומת
        if len < buf.len() {
            buf[len] = b'.';
            len += 1;
        }

        // העתקת הסיומת
        for &b in disk_ext.as_bytes() {
            if len < buf.len() {
                buf[len] = b;
                len += 1;
            }
        }

        // הוספת פסיק מפריד בין קבצים
        if len < buf.len() {
            buf[len] = b',';
            len += 1;
        }
    }

    buf
}

// צור תיקייה חדשה ב-root
pub fn create_dir(name: &str) -> bool {
    let (fat_start, root_start, data_start, spc) = get_layout();

    let cluster = match find_free_cluster(fat_start) {
        Some(c) => c,
        None => return false,
    };

    mark_cluster_used(fat_start, cluster);

    // אתחל את ה-cluster של התיקייה לאפסים
    let empty = [0u8; 512];
    let sector = data_start + (cluster as u32 - 2) * spc;
    ata::write_sector(sector, &empty);

    // כתוב entry ב-root
    write_dir_entry(root_start, name, cluster, true)
}

// כתוב קובץ בתוך תיקייה
pub fn create_file_in(dir: &str, name: &str, ext: &str, content: &[u8]) -> bool {
    let (fat_start, root_start, data_start, spc) = get_layout();

    // מצא את ה-cluster של התיקייה
    let dir_cluster = match find_entry(root_start, dir, "", true) {
        Some(c) => c,
        None => return false,
    };

    // מצא cluster פנוי לתוכן
    let cluster = match find_free_cluster(fat_start) {
        Some(c) => c,
        None => return false,
    };

    // כתוב תוכן
    let mut data = [0u8; 512];
    let len = content.len().min(512);
    data[..len].copy_from_slice(&content[..len]);
    let sector = data_start + (cluster as u32 - 2) * spc;
    ata::write_sector(sector, &data);

    mark_cluster_used(fat_start, cluster);

    // כתוב entry בתוך התיקייה
    let dir_sector = data_start + (dir_cluster as u32 - 2) * spc;
    write_file_entry(dir_sector, name, ext, cluster, len)
}

// קרא קובץ מתוך תיקייה
pub fn read_file_in(dir: &str, name: &str, ext: &str, buf: &mut [u8; 512]) -> u32 {
    let (_, root_start, data_start, spc) = get_layout();

    let dir_cluster = match find_entry(root_start, dir, "", true) {
        Some(c) => c,
        None => return 0,
    };

    let dir_sector = data_start + (dir_cluster as u32 - 2) * spc;

    match find_entry(dir_sector, name, ext, false) {
        Some(cluster) => {
            let sector = data_start + (cluster as u32 - 2) * spc;
            ata::read_sector(sector, buf);
            512 // או תחזיר size אמיתי אם שמרת אותו
        }
        None => 0,
    }
}

// --- פונקציות עזר פנימיות ---

// חפש entry לפי שם, החזר את ה-cluster שלו
fn find_entry(sector: u32, name: &str, ext: &str, is_dir: bool) -> Option<u16> {
    let mut buf = [0u8; 512];
    ata::read_sector(sector, &mut buf);

    for i in 0..16usize {
        let e = &buf[i * 32..(i + 1) * 32];
        if e[0] == 0x00 {
            break;
        }
        if e[0] == 0xE5 {
            continue;
        }

        let attr = e[11];
        let entry_is_dir = attr & 0x10 != 0;
        if entry_is_dir != is_dir {
            continue;
        }

        let ename = core::str::from_utf8(&e[0..8]).unwrap_or("").trim_end();
        let eext = core::str::from_utf8(&e[8..11]).unwrap_or("").trim_end();

        if ename.eq_ignore_ascii_case(name) && eext.eq_ignore_ascii_case(ext) {
            return Some(u16::from_le_bytes([e[26], e[27]]));
        }
    }
    None
}

// כתוב directory entry (לתיקייה)
fn write_dir_entry(sector: u32, name: &str, cluster: u16, is_dir: bool) -> bool {
    let mut buf = [0u8; 512];
    ata::read_sector(sector, &mut buf);

    for i in 0..16usize {
        if buf[i * 32] == 0x00 {
            let mut entry = [0x20u8; 32];

            let nb = name.as_bytes();
            let nl = nb.len().min(8);
            entry[..nl].copy_from_slice(&nb[..nl]);

            entry[11] = if is_dir { 0x10 } else { 0x00 };
            entry[26] = cluster as u8;
            entry[27] = (cluster >> 8) as u8;

            buf[i * 32..(i + 1) * 32].copy_from_slice(&entry);
            ata::write_sector(sector, &buf);
            return true;
        }
    }
    false
}

// כתוב file entry (לקובץ)
fn write_file_entry(sector: u32, name: &str, ext: &str, cluster: u16, size: usize) -> bool {
    let mut buf = [0u8; 512];
    ata::read_sector(sector, &mut buf);

    for i in 0..16usize {
        if buf[i * 32] == 0x00 {
            let mut entry = [0x20u8; 32];

            let nb = name.as_bytes();
            entry[..nb.len().min(8)].copy_from_slice(&nb[..nb.len().min(8)]);

            let eb = ext.as_bytes();
            entry[8..8 + eb.len().min(3)].copy_from_slice(&eb[..eb.len().min(3)]);

            entry[11] = 0x00;
            entry[26] = cluster as u8;
            entry[27] = (cluster >> 8) as u8;
            entry[28] = size as u8;
            entry[29] = (size >> 8) as u8;

            buf[i * 32..(i + 1) * 32].copy_from_slice(&entry);
            ata::write_sector(sector, &buf);
            return true;
        }
    }
    false
}

// מצא cluster של נתיב כמו "DOCS/SRC"
fn resolve_path(path: &str) -> Option<u32> {
    let (_, root_start, data_start, spc) = get_layout();

    let mut current_sector = root_start;

    for part in path.split('/') {
        if part.is_empty() {
            continue;
        }

        let cluster = find_entry(current_sector, part, "", true)?;
        current_sector = data_start + (cluster as u32 - 2) * spc;
    }

    Some(current_sector)
}

pub fn create_dir_at(path: &str, name: &str) -> bool {
    let (fat_start, _, data_start, spc) = get_layout();
    let parent_sector = match resolve_path(path) {
        Some(s) => s,
        None => return false,
    };

    let cluster = match find_free_cluster(fat_start) {
        Some(c) => c,
        None => return false,
    };

    mark_cluster_used(fat_start, cluster);
    let empty = [0u8; 512];
    ata::write_sector(data_start + (cluster as u32 - 2) * spc, &empty);

    write_dir_entry(parent_sector, name, cluster, true)
}

pub fn create_file_at(path: &str, name: &str, ext: &str, content: &[u8]) -> bool {
    let (fat_start, _, data_start, spc) = get_layout();
    let parent_sector = match resolve_path(path) {
        Some(s) => s,
        None => return false,
    };

    let cluster = match find_free_cluster(fat_start) {
        Some(c) => c,
        None => return false,
    };

    let mut data = [0u8; 512];
    let len = content.len().min(512);
    data[..len].copy_from_slice(&content[..len]);
    ata::write_sector(data_start + (cluster as u32 - 2) * spc, &data);
    mark_cluster_used(fat_start, cluster);

    write_file_entry(parent_sector, name, ext, cluster, len)
}

pub fn read_file_at(path: &str, name: &str, ext: &str, buf: &mut [u8; 512]) -> u32 {
    let (_, _, data_start, spc) = get_layout();
    let parent_sector = match resolve_path(path) {
        Some(s) => s,
        None => return 0,
    };

    match find_entry(parent_sector, name, ext, false) {
        Some(cluster) => {
            ata::read_sector(data_start + (cluster as u32 - 2) * spc, buf);
            512
        }
        None => 0,
    }
}
