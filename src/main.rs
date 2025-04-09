mod app;
mod ui;
mod commands;
mod execute;
mod cli;

use ratatui:: { backend::CrosstermBackend, Terminal};
use crossterm::{
    terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
    event::{EnableMouseCapture, DisableMouseCapture},
};
use std::{
    env,
    error::Error,
    io::{self},
    io::Write
};

use crate::cli::text_flow;

fn default_flow() -> Result<(), Box<dyn Error>> {
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

fn main() -> Result<(), Box<dyn Error>> {
   let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        text_flow();
    } else {
        default_flow();
    }

   Ok(())
}