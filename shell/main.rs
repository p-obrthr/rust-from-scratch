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
            execute(&cmd, &args);
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

fn parse_input(input: &str) -> Option<(String, Vec<String>)> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut chars = input.chars().peekable();
    let mut single_quote_mode = false;
    let mut double_quote_mode = false;

    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                if single_quote_mode {
                    current.push('\\');
                } else if let Some(next_char) = chars.next() {
                    if double_quote_mode {
                        match next_char {
                            '\\' | '"' | '$' | '`' => current.push(next_char),
                            _ => {
                                current.push('\\');
                                current.push(next_char);
                            }
                        }
                    } else {
                        current.push(next_char);
                    }
                }
            }

            '\'' => {
                if !double_quote_mode {
                    single_quote_mode = !single_quote_mode;
                } else {
                    current.push(c);
                }
            }

            '"' => {
                if !single_quote_mode {
                    double_quote_mode = !double_quote_mode;
                } else {
                    current.push(c);
                }
            }

            c if c.is_whitespace() && !single_quote_mode && !double_quote_mode => {
                if !current.is_empty() {
                    args.push(current.clone());
                    current.clear();
                }
                while let Some(next) = chars.peek() {
                    if next.is_whitespace() {
                        chars.next();
                    } else {
                        break;
                    }
                }
            }

            _ => {
                current.push(c);
            }
        }
    }

    if !current.is_empty() {
        args.push(current);
    }

    if args.is_empty() {
        None
    } else {
        let cmd = args.remove(0);
        Some((cmd, args))
    }
}

fn execute(cmd: &str, args: &[String]) {
    let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    match cmd {
        "echo" => execute_echo(&args_str),
        "exit" => execute_exit(&args_str),
        "type" => execute_type(&args_str, BUILT_INS),
        "pwd" => execute_pwd(),
        "cd" => execute_cd(&args_str),
        _ => {
            if !execute_external(cmd, &args_str) {
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

const BUILT_INS: &[&str] = &["echo", "exit", "type", "pwd", "cd"];

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

fn execute_pwd() {
    match env::current_dir() {
        Ok(path) => println!("{}", path.display()),
        Err(e) => eprintln!("Err getting current dir: {}", e),
    }
}

fn execute_cd(args: &[&str]) {
    let target = if args[0] == "~" {
        env::var("HOME").unwrap_or_else(|_| String::from("/"))
    } else {
        args[0].to_string()
    };

    let path = Path::new(&target);

    if let Err(_) = env::set_current_dir(path) {
        eprintln!("cd: {}: No such file or directory", target);
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
