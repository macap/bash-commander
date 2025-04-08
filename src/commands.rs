








use std::io::{self, BufReader, BufRead, Write};
use std::fs;
use std::path::PathBuf;
use dirs;

use crate::app::BashCmd;

pub fn get_commands_file_path() -> PathBuf {
    let mut path = PathBuf::new();
    if let Some(home_dir) = dirs::home_dir() {
        path.push(home_dir);
        path.push(".config");
        path.push("bash_command_app");
        fs::create_dir_all(&path).unwrap_or_else(|_| {});
        path.push("commands.txt"); 
    }
    path
}

pub fn load_commands_from_file() -> io::Result<Vec<BashCmd>> {
    let path = get_commands_file_path();
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = fs::File::open(path)?; 
    let reader = BufReader::new(file); 

    let mut commands = Vec::new();
    for line_result in reader.lines() { 
        let line = line_result?; 
        let parts: Vec<&str> = line.splitn(3, '※').collect(); 
        if parts.len() == 3 { 
            let name = parts[0].trim().to_string(); 
            let desc = parts[1].trim().to_string(); 
            let command_text = parts[2].trim().to_string(); 

            
            let bash_cmd = BashCmd {
                name,
                desc,
                command: command_text,
                index: (commands.len() + 1) as u8, 
                category: 1, 
                favourite: false, 
            };
            commands.push(bash_cmd); 
        }
        
    }
    Ok(commands)
}

pub fn save_commands_to_file(app: &crate::app::App) -> io::Result<()> {
    let path = get_commands_file_path();
    let mut file = fs::File::create(path)?; 
    for command in &app.items { 
        writeln!(file, "{name}※{desc}※{command}", 
                 name = command.name,
                 desc = command.desc,
                 command = command.command)?;
    }
    Ok(())
}

pub fn append_command_to_file(command: &BashCmd) -> io::Result<()> {
    let path = get_commands_file_path();
    let mut file = fs::OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(path)?;

    writeln!(file, "{name}※{desc}※{command}", 
                 name = command.name,
                 desc = command.desc,
                 command = command.command)?;

    Ok(())
}
