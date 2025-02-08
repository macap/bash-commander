

use std::process::{Command, Stdio}; 
use std::io::{self, Result}; 


use std::os::unix::process::CommandExt;


pub fn execute_command(command_string_option: Option<String>) -> Result<()> { 
    if let Some(command_to_execute) = command_string_option {
        let parts: Vec<&str> = command_to_execute.split_whitespace().collect();

        if let Some(command_name) = parts.get(0) {
            let mut command = Command::new(command_name);

            for arg in parts.iter().skip(1) {
                command.arg(arg);
            }
            command.exec();
        } else {
            eprintln!("No command provided");
        }
    }
     Ok(())
}