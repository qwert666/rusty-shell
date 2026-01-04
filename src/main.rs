#[allow(unused_imports)]
use std::io::{self, Write};
use std::env;
use std::path::Path;
use std::process::Command;
use std::os::unix::process::CommandExt;

const BUILTINS: &[&str] = &["echo", "exit", "type", "pwd", "cd"];


fn main() {
    loop {
        match read_command() {
            Ok(command) => {
                if !execute_command(&command) {
                    break;
                }
            }
            Err(_) => break,
        }
    }
}

fn read_command() -> Result<String, std::io::Error> {
    print!("$ ");
    io::stdout().flush()?;
    let mut command = String::new();
    io::stdin().read_line(&mut command)?;
    Ok(command.trim().to_string())
}

fn execute_command(command: &str) -> bool {
    match command {
        "exit" => false,
        cmd => {
            let parts: Vec<&str> = cmd.split_whitespace().collect();
            match parts.as_slice() {
                ["echo", args @ ..] => handle_echo(args),
                ["type", args @ ..] => handle_type(args),
                ["cd", args @ ..] => handle_cd(args),
                ["pwd"] => handle_pwd(),
                [] => {}
                [command, args @ ..] => handle_external_command(command, args),
            }
            true
        }
    }
}

fn handle_pwd() {
    if let Ok(path) = env::current_dir() {
        println!("{}", path.display());
    }
}

fn handle_cd(args: &[&str]) {
    let target = if args.is_empty() {
        env::var("HOME").unwrap_or_else(|_| "/".to_string())
    } else {
        args[0].to_string()
    };
    
    if let Err(_) = env::set_current_dir(&target) {
        println!("cd: {}: No such file or directory", target);
    }
}

fn handle_external_command(command: &str, args: &[&str]) {
    if let Some(path) = find_in_path(command) {
        Command::new(path)
            .arg0(command)
            .args(args)
            .spawn()
            .expect("command failed to start")
            .wait()
            .expect("command wasn't running");
    } else {
        println!("{}: command not found", command);
    }
}

fn handle_echo(args: &[&str]) {
    println!("{}", args.join(" "));
}

fn is_builtin(cmd: &str) -> bool {
    BUILTINS.contains(&cmd)
}

fn find_in_path(cmd: &str) -> Option<std::path::PathBuf> {
    env::var("PATH").ok()?.split(':').find_map(|dir| {
        let full_path = Path::new(dir).join(cmd);
        if full_path.is_file() && is_executable(&full_path) {
            Some(full_path)
        } else {
            None
        }
    })
}

fn handle_type(args: &[&str]) {
    for arg in args {
        if is_builtin(arg) {
            println!("{} is a shell builtin", arg);
        } else if let Some(path) = find_in_path(arg) {
            println!("{} is {}", arg, path.display());
        } else {
            println!("{}: not found", arg);
        }
    }
}

fn is_executable(path: &Path) -> bool {
    {
        use std::os::unix::fs::PermissionsExt;
        path.metadata()
            .map(|m| (m.permissions().mode() & 0o111) != 0)
            .unwrap_or(false)
    }
}
