use x86_64::instructions::port::Port;

pub fn read_sector(sector: u32, buf: &mut [u8; 512]) {
    unsafe {
        // שלב 1: בחר master drive + LBA mode
        Port::<u8>::new(0x1F6).write(0xE0 | ((sector >> 24) as u8 & 0x0F));

        // שלב 2: כמה sectors לקרוא
        Port::<u8>::new(0x1F2).write(1);

        // שלב 3: מספר ה-sector
        Port::<u8>::new(0x1F3).write(sector as u8);
        Port::<u8>::new(0x1F4).write((sector >> 8) as u8);
        Port::<u8>::new(0x1F5).write((sector >> 16) as u8);

        // שלב 4: פקודת READ
        Port::<u8>::new(0x1F7).write(0x20);

        // שלב 5: חכה שמוכן
        let mut tries = 0u32;
        loop {
            let status = Port::<u8>::new(0x1F7).read();
            if status & 0x08 != 0 {
                break;
            }
            tries += 1;
            if tries > 100_000 {
                break;
            } // timeout - אל תנעל לנצח
        }

        // שלב 6: קרא 512 בייטים
        let mut data_port = Port::<u16>::new(0x1F0);
        for i in 0..256 {
            let word = data_port.read();
            buf[i * 2] = word as u8;
            buf[i * 2 + 1] = (word >> 8) as u8;
        }
    }
}

pub fn write_sector(sector: u32, buf: &[u8; 512]) {
    unsafe {
        // שלב 1: בחר master drive + LBA mode
        Port::<u8>::new(0x1F6).write(0xE0 | ((sector >> 24) as u8 & 0x0F));

        // שלב 2: כמה sectors לכתוב
        Port::<u8>::new(0x1F2).write(1);

        // שלב 3: מספר ה-sector
        Port::<u8>::new(0x1F3).write(sector as u8);
        Port::<u8>::new(0x1F4).write((sector >> 8) as u8);
        Port::<u8>::new(0x1F5).write((sector >> 16) as u8);

        // שלב 4: פקודת WRITE
        Port::<u8>::new(0x1F7).write(0x30);

        // שלב 5: חכה שמוכן
        let mut tries = 0u32;
        loop {
            let status = Port::<u8>::new(0x1F7).read();
            if status & 0x08 != 0 {
                break;
            }
            tries += 1;
            if tries > 100_000 {
                break;
            } // timeout - אל תנעל לנצח
        }

        // שלב 6: כתוב 512 בייטים
        let mut data_port = Port::<u16>::new(0x1F0);
        for i in 0..256 {
            let word = buf[i * 2] as u16 | ((buf[i * 2 + 1] as u16) << 8);
            data_port.write(word);
        }

        // שלב 7: חכה שהכתיבה תסתיים
        let mut tries = 0u32;
        loop {
            let status = Port::<u8>::new(0x1F7).read();
            if status & 0x80 == 0 {
                // BSY bit נכבה = סיים
                break;
            }
            tries += 1;
            if tries > 100_000 {
                break;
            }
        }

        // שלב 8: cache flush
        Port::<u8>::new(0x1F7).write(0xE7);
        loop {
            let status = Port::<u8>::new(0x1F7).read();
            if status & 0x80 == 0 {
                break;
            }
        }
    }
}
