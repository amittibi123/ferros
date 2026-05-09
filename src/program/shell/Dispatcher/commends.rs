use crate::WRITER;

pub fn command_echo(args: &str) {
    // כאן אתה משתמש ב-print! של הקרנל שלך
    crate::WRITER.get().unwrap().lock().println(args);
}

pub fn command_help(_args: &str) {
    crate::WRITER.get().unwrap().lock().println("Available commands: echo, clear, help");
}

pub fn clear(_args: &str) {
    crate::WRITER.get().unwrap().lock().clear_screen();
}