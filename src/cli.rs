
use std::env;
use std::io::{self, Write};

use crate::commands::append_command_to_file;
use crate::app::BashCmd;

pub fn get_user_input(label: &str) -> String {
    print!("{}:", label);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    return input.trim().to_string();
}

pub fn text_flow() {
    let args: Vec<String> = env::args().collect();

    let command_text: String = args[1..].join(" ");

    println!("Command: {}", command_text);
    let name = get_user_input("Name");
    let description = get_user_input("Description");

    let bash_cmd = BashCmd {
        name,
        desc: description,
        command: command_text,
        index: 100, 
        category: 1, 
        favourite: false, 
    };

    let confirmation = get_user_input("Do you want to save this command? (y/n)");
    
    if confirmation.to_lowercase() == "y" {
            append_command_to_file(&bash_cmd);

    println!("command saved.");
    } else {
        println!("Command not saved.");
        return;
    }


}