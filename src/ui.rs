use ratatui::{
    Terminal,
    widgets::{List, ListItem, Block, Borders, Paragraph, BorderType, Clear},
    layout::{Layout, Constraint, Direction, Rect, Alignment},
    style::{Style, Color, Modifier},
    text::{Span, Text, Line},
};
use crossterm::event::{Event, KeyCode, KeyModifiers};
use std::io;

use crate::app::BashCmd;
use crate::commands::save_commands_to_file; 



fn draw_command_details(f: &mut ratatui::Frame, command: Option<&BashCmd>, area: Rect) {
    f.render_widget(Block::default().borders(Borders::ALL).title("Details").border_type(BorderType::Rounded), area);

    let details_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3), 
            Constraint::Length(3), 
            Constraint::Length(3), 
            Constraint::Length(3), 
            Constraint::Min(0),    
        ].as_ref())
        .split(area);

    if let Some(cmd) = command { 
        let name_paragraph = Paragraph::new(Text::from(Line::from(vec![Span::styled("Name: ", Style::default().add_modifier(Modifier::BOLD)), Span::raw(cmd.name.as_str())])))
            .block(Block::default().borders(Borders::BOTTOM));
        f.render_widget(name_paragraph, details_layout[0]);

        let desc_paragraph = Paragraph::new(Text::from(Line::from(vec![Span::styled("Desc: ", Style::default().add_modifier(Modifier::BOLD)), Span::raw(cmd.desc.as_str())])))
            .block(Block::default().borders(Borders::BOTTOM));
        f.render_widget(desc_paragraph, details_layout[1]);

        let command_paragraph = Paragraph::new(Text::from(Line::from(vec![Span::styled("Command: ", Style::default().add_modifier(Modifier::BOLD)), Span::raw(cmd.command.as_str())])))
            .block(Block::default().borders(Borders::BOTTOM));
        f.render_widget(command_paragraph, details_layout[2]);

        let category_paragraph = Paragraph::new(Text::from(Line::from(vec![Span::styled("Category: ", Style::default().add_modifier(Modifier::BOLD)), Span::raw(cmd.category.to_string())]))) 
            .block(Block::default());
        f.render_widget(category_paragraph, details_layout[3]);

    } else { 
        let placeholder_text = Text::from("Select command from the list\nto see details here");
        let placeholder_paragraph = Paragraph::new(placeholder_text)
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::NONE).title("Details").border_type(BorderType::Rounded));
        f.render_widget(placeholder_paragraph, area); 
    }
}


fn draw_add_popup(f: &mut ratatui::Frame, app: &crate::app::App) {
    let popup_title = if app.is_editing { "Edit command" } else { "Add command" };
    
    let block = Block::default().title(popup_title).borders(Borders::ALL).border_type(BorderType::Rounded);
    let popup_area = left_aligned_rect(60, 40, f.size());
    f.render_widget(Clear, popup_area);
    f.render_widget(block, popup_area);

    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3), 
            Constraint::Length(3), 
            Constraint::Length(3), 
            Constraint::Min(0),    
        ].as_ref())
        .split(popup_area);

      
    let focused_style = Style::default().fg(Color::LightGreen).add_modifier(Modifier::BOLD);
    let normal_style = Style::default();

    
    let name_block = Block::default().borders(Borders::ALL).title("Name")
        .border_style(if app.popup_input_focused == 0 { focused_style } else { normal_style }); 
    let name_paragraph = Paragraph::new(Text::from(app.popup_input_name.as_str()))
        .block(name_block);
    f.render_widget(name_paragraph, popup_layout[0]);

    
    let desc_block = Block::default().borders(Borders::ALL).title("Description")
        .border_style(if app.popup_input_focused == 1 { focused_style } else { normal_style }); 
    let desc_paragraph = Paragraph::new(Text::from(app.popup_input_desc.as_str()))
        .block(desc_block);
    f.render_widget(desc_paragraph, popup_layout[1]);

    
    let command_block = Block::default().borders(Borders::ALL).title("Command")
        .border_style(if app.popup_input_focused == 2 { focused_style } else { normal_style }); 
    let command_paragraph = Paragraph::new(Text::from(app.popup_input_command.as_str()))
        .block(command_block);
    f.render_widget(command_paragraph, popup_layout[2]);
}


fn left_aligned_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_width = r.width * percent_x / 100;
    let popup_height = r.height * percent_y / 100;

    Rect {
        x: r.left() + 2, 
        y: r.top() + 2,  
        width: popup_width,
        height: popup_height,
    }
}


pub fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, mut app: crate::app::App) -> io::Result<Option<String>> {
    loop {
        terminal.draw(|f| {
            let main_layout = Layout::default() 
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(3),      
                        Constraint::Percentage(90), 
                        Constraint::Length(2),      
                    ].as_ref()
                )
                .split(f.size());

            let content_layout = Layout::default() 
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(50), 
                    Constraint::Percentage(50), 
                ].as_ref())
                .split(main_layout[1]); 


            let input_paragraph = Paragraph::new(Text::from(app.filter_text.as_str()))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title("Filter")
                    );
            f.render_widget(input_paragraph, main_layout[0]);


            
            let items: Vec<ListItem> = app.filtered_items
                .iter()
                .map(|item| {
                    ListItem::new(Span::styled(item.name.clone(), Style::default().fg(Color::Gray)))
                })
                .collect();

            let list = List::new(items)
                .block(Block::default().title("Commands").borders(Borders::ALL))
                .highlight_style(Style::default().add_modifier(Modifier::BOLD).fg(Color::Gray).bg(Color::Blue))
                .highlight_symbol("> ");

            f.render_stateful_widget(list, content_layout[0], &mut app.state);

             
            let selected_command = match app.state.selected() { 
                Some(index) => app.filtered_items.get(index),
                None => None,
            };
            draw_command_details(f, selected_command, content_layout[1]); 


            
            if app.show_popup {
                draw_add_popup(f, &app);
            }

                let help_text = Text::from(Line::from(vec![ 
                Span::styled("ESC / Ctrl+Q", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(": Exit"),
                Span::raw(" | "), 
                Span::styled("Ctrl+A", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(": Add command"),
                Span::raw(" | "), 
                Span::styled("Ctrl+E", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(": Edit command"),
                
            ]));
            let help_paragraph = Paragraph::new(help_text)
                .style(Style::default().fg(Color::Gray)) 
                .alignment(Alignment::Center) 
                .block(Block::default().style(Style::default().bg(Color::DarkGray))); 

            f.render_widget(help_paragraph, main_layout[2]); 
   
        })?;

        if let Event::Key(key) = crossterm::event::read()? {
            
            if !app.show_popup { 
                match key.code {
                    KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        save_commands_to_file(&app)?; 
                        return Ok(None)
                    }, 
                    KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => app.show_add_popup(), 
                    KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => { 
                        if let Some(selected_index) = app.state.selected() { 
                            app.enter_edit_mode(selected_index); 
                        }
                    },
                    KeyCode::Enter => {
                       if let Some(selected_index) = app.state.selected() {
                            if let Some(selected_command) = app.filtered_items.get(selected_index) {
                                
                                app.selected_command_to_execute = Some(selected_command.command.clone()); 
                                save_commands_to_file(&app)?; 
                                return Ok(app.selected_command_to_execute.clone()); 
                            } else {
                                app.selected_command_to_execute = None;
                                return Ok(None); 
                            }
                        } else {
                            app.selected_command_to_execute = None; 
                            
                        }
                    }
                    KeyCode::Char(ch) => {
                        app.add_char_to_filter(ch);
                    }
                    KeyCode::Backspace => {
                        app.remove_char_from_filter();
                    }
                    KeyCode::Down => app.next(),
                    KeyCode::Up => app.previous(),
                    KeyCode::Esc => {
                        save_commands_to_file(&app)?; 
                        return Ok(None)
                    }
                    _ => {}
                }
            } else { 
               match key.code {
                    KeyCode::Esc => app.hide_add_popup(),
                    KeyCode::Enter => app.save_command(),
                    KeyCode::Tab => app.next_popup_input_focus(), 
                    KeyCode::Char(ch) => { 
                        match app.popup_input_focused {
                            0 => app.add_char_to_popup_input_name(ch), 
                            1 => app.add_char_to_popup_input_desc(ch), 
                            2 => app.add_char_to_popup_input_command(ch), 
                            _ => {} 
                        }
                    }
                    KeyCode::Backspace => { 
                        match app.popup_input_focused {
                            0 => app.remove_char_from_popup_input_name(), 
                            1 => app.remove_char_from_popup_input_desc(), 
                            2 => app.remove_char_from_popup_input_command(), 
                            _ => {} 
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
