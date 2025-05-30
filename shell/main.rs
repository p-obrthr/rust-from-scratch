use std::env;
use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{self, Command, Stdio};

fn main() {
    loop {
        let input = match read_input() {
            Ok(line) => line,
            Err(e) => {
                eprintln!("Err reading input: {}", e);
                continue;
            }
        };

        if let Some((cmd, args)) = parse_input(&input) {
            execute(cmd, &args);
        }
    }
}

fn read_input() -> io::Result<String> {
    print!("$ ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input)
}

fn parse_input(input: &str) -> Option<(&str, Vec<&str>)> {
    let mut parts = input.trim().split_whitespace();
    parts.next().map(|cmd| (cmd, parts.collect()))
}

fn execute(cmd: &str, args: &[&str]) {
    match cmd {
        "echo" => execute_echo(args),
        "exit" => execute_exit(args),
        "type" => execute_type(args, BUILT_INS),
        _ => {
            if !execute_external(cmd, args) {
                handle_not_found(cmd);
            }
        }
    }
}

fn execute_echo(args: &[&str]) {
    println!("{}", args.join(" "));
}

fn execute_exit(args: &[&str]) {
    let exit_code = if args.is_empty() {
        0
    } else {
        args[0].parse::<i32>().unwrap_or(1)
    };

    process::exit(exit_code);
}

const BUILT_INS: &[&str] = &["echo", "exit", "type"];

fn execute_type(args: &[&str], built_ins: &[&str]) {
    let cmd = args[0];

    if built_ins.contains(&cmd) {
        println!("{} is a shell builtin", cmd);
        return;
    }

    if let Some(path) = find_executable(cmd) {
        println!("{} is {}", cmd, path.display());
    } else {
        println!("{}: not found", cmd);
    }
}

fn execute_external(cmd: &str, args: &[&str]) -> bool {
    if let Some(_) = find_executable(cmd) {
        let cmd_name = Path::new(cmd)
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new(cmd));

        let mut child = match Command::new(cmd_name)
            .args(args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
        {
            Ok(c) => c,
            Err(_) => return false,
        };

        let _ = child.wait();
        true
    } else {
        false
    }
}

fn find_executable(cmd: &str) -> Option<std::path::PathBuf> {
    let path_var = env::var("PATH").ok()?;

    for dir in path_var.split(':') {
        let cmd_path = Path::new(dir).join(cmd);

        let metadata = match fs::metadata(&cmd_path) {
            Ok(m) => m,
            Err(_) => continue,
        };

        if !metadata.is_file() || (metadata.permissions().mode() & 0o111) == 0 {
            continue;
        }

        return Some(cmd_path);
    }

    None
}

fn handle_not_found(cmd: &str) {
    println!("{}: command not found", cmd);
}
