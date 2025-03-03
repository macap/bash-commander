mod app;
mod ui;
mod commands;
mod execute;

use ratatui:: { backend::CrosstermBackend, Terminal};
use crossterm::{
    terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
    event::{EnableMouseCapture, DisableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers},
};
use std::{
    error::Error,
    io::{self, BufReader, BufRead, stdout},
    vec,
    fs,
    path::PathBuf,
    io::Write,
    process::{Command, Stdio}
};


#[macro_use] extern crate run_shell;

use crate::app::BashCmd;

use crate::commands::save_commands_to_file; 



fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = app::App::new();
    let res = ui::run_app(&mut terminal, app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    let selected_command_string_option = match res { 
        Ok(command_string_option) => command_string_option, 
        Err(err) => {
            println!("{:?}", err);
            None 
        }
    };

    

        execute::execute_command(selected_command_string_option); 

        

    Ok(())
}