use crate::program::shell::Dispatcher::commends::rmdir;

pub(crate) mod commends; // מצהיר על קיום המודול

pub fn dispatch_command(input: &str, dir: &mut heapless::String<64>) {
    let mut parts = input.splitn(2, ' ');
    let cmd = parts.next().unwrap_or("");
    let args = parts.next().unwrap_or("");

    match cmd {
        // משתמשים ב-:: כדי לגשת לפונקציה בתוך המודול
        "ECHO" => commends::command_echo(args),
        "HELP" => commends::command_help(args),
        "CLEAR" => commends::clear(args),
        "DISKTEST" => commends::command_disktest(args),
        "WRITE" => commends::command_write(args, dir),
        "READ" => commends::command_read(args, dir),
        "DELETE" => commends::command_delete(args, dir),
        "LS" => commends::commeand_list(dir),
        "MKDIR" => commends::mkdir(args, dir),
        "RMDIR" => commends::rmdir(args, dir),
        "CD" => commends::cd(args, dir),
        "SWITCH" => commends::switch(),
        "" => {}
        _ => commends::command_echo("errore commend not found"),
    }
}
