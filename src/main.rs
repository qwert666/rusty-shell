#[allow(unused_imports)]
use std::io::{self, Write};
use std::env;
use std::path::Path;
use std::process::Command;
use std::os::unix::process::CommandExt;
use std::fs::File;

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
            let parts = parse_command(cmd);
            let (cmd_parts, output_file) = extract_redirection(&parts);


            match cmd_parts.as_slice() {
                [cmd, args @ ..] if cmd == "echo" => handle_echo(args, output_file.as_deref()),
                [cmd, args @ ..] if cmd == "type" => handle_type(args),
                [cmd, args @ ..] if cmd == "cd" => handle_cd(args),
                [cmd] if cmd == "pwd" => handle_pwd(),
                [] => {}
                [command, args @ ..] => handle_external_command(command, args),
            }
            true
        }
    }
}

fn extract_redirection(parts: &[String]) -> (Vec<String>, Option<String>) {
    if let Some(pos) = parts.iter().position(|p| p == ">" || p == "1>") {
        let cmd_parts = parts[..pos].to_vec();
        let output_file = parts.get(pos + 1).cloned();
        (cmd_parts, output_file)
    } else {
        (parts.to_vec(), None)
    }
}

fn parse_command(input: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut chars = input.chars();
    
    while let Some(ch) = chars.next() {
        let is_unquoted = !in_single_quote && !in_double_quote;

        match ch {
            '\\' if is_unquoted => {
                if let Some(next_ch) = chars.next() {
                    current.push(next_ch);
                }
            }
            '\\' if in_double_quote => {
                if let Some(next_ch) = chars.next() {
                    match next_ch {
                        '"' | '\\' => current.push(next_ch),
                        _ => {
                            current.push('\\');
                            current.push(next_ch);
                        }
                    }
                }
            }
            '\'' if !in_double_quote => {
                in_single_quote = !in_single_quote;
            }
            ' ' | '\t' if is_unquoted => {
                if !current.is_empty() {
                    parts.push(current);
                    current = String::new();
                }
            }
            '\"' if !in_single_quote => {
                in_double_quote = !in_double_quote;
            }
            _ => {
                current.push(ch);
            }
        }
    }
    
    if !current.is_empty() {
        parts.push(current);
    }
    
    parts
}

fn handle_pwd() {
    if let Ok(path) = env::current_dir() {
        println!("{}", path.display());
    }
}

fn handle_cd(args: &[String]) {
    let target = if args.is_empty() || args[0] == "~" {
        env::var("HOME").unwrap_or_else(|_| "/".to_string())
    } else {
        args[0].to_string()
    };
    
    if let Err(_) = env::set_current_dir(&target) {
        println!("cd: {}: No such file or directory", target);
    }
}

fn handle_external_command(command: &str, args: &[String]) {
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

fn handle_echo(args: &[String], output_file: Option<&str>) {
    let output = args.join(" ");
    write_output(&output, output_file);
}

fn write_output(content: &str, output_file: Option<&str>) {
    if let Some(file_path) = output_file {
        match File::create(file_path) {
            Ok(mut file) => {
                writeln!(file, "{}", content).ok();
            }
            Err(e) => {
                eprintln!("Error writing to {}: {}", file_path, e);
            }
        }
    } else {
        println!("{}", content);
    }
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

fn handle_type(args: &[String]) {
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
    use std::os::unix::fs::PermissionsExt;
    path.metadata()
        .map(|m| (m.permissions().mode() & 0o111) != 0)
        .unwrap_or(false)
}