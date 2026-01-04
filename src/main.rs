#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut command = String::new();
        match io::stdin().read_line(&mut command) {
            Ok(_) => {
                let command = command.trim();
                if command == "exit" {
                    break;
                }
                println!("{}: command not found", command);
            }
            Err(_) => break,
        }
    }
}
