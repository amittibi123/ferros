mod commends; // מצהיר על קיום המודול

pub fn dispatch_command(input: &str) {
    let mut parts = input.splitn(2, ' ');
    let cmd = parts.next().unwrap_or("");
    let args = parts.next().unwrap_or("");

    match cmd {
        // משתמשים ב-:: כדי לגשת לפונקציה בתוך המודול
        "ECHO" => commends::command_echo(args),
        "HELP" => commends::command_help(args),
        "CLEAR" => commends::clear(args),
        "" => {}, 
        _ => commends::command_echo("errore"),
    }
}