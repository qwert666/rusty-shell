#[allow(unused_imports)]
use std::io::{self, Write};

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
                [] => {}
                _ => println!("{}: command not found", cmd),
            }
            true
        }
    }
}

fn handle_echo(args: &[&str]) {
    println!("{}", args.join(" "));
}
