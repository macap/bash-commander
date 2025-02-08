use ratatui::widgets::ListState; 

use crate::commands::load_commands_from_file; 


#[derive(Clone)]
pub struct BashCmd {
    pub name: String,
    pub desc: String,
    pub command: String,
    pub index: u8,
    pub category: u8,
    pub favourite: bool,
}





pub struct App {
    pub items: Vec<BashCmd>,
    pub filtered_items: Vec<BashCmd>,
    pub state: ListState,
    pub filter_text: String,
    pub show_popup: bool,
    pub popup_input_name: String,
    pub popup_input_desc: String,
    pub popup_input_command: String,
    pub popup_input_focused: u8, 
    pub is_editing: bool,               
    pub editing_command_index: Option<usize>, 
    pub selected_command_to_execute: Option<String>, 
}



impl App {
    pub fn new() -> App {
         let initial_items = vec![
            BashCmd { name: "ls".to_string(), desc: "List files".to_string(), command: "ls -l".to_string(), index: 1, category: 1, favourite: false },
         ];

        let loaded_commands = load_commands_from_file().unwrap_or_else(|_| Vec::new()); 
        let items_to_use = if !loaded_commands.is_empty() { 
            loaded_commands
        } else { 
            initial_items
        };

        App {
            items: items_to_use.clone(),
            filtered_items: items_to_use.clone(),
            state: ListState::default(),
            filter_text: String::new(),
            show_popup: false, 
            popup_input_name: String::new(),    
            popup_input_desc: String::new(),    
            popup_input_command: String::new(), 
            popup_input_focused: 0, 
            is_editing: false,                
            editing_command_index: None,         
            selected_command_to_execute: None, 
            
        }
    }

    
    fn update_filtered_items(&mut self) {
        self.filtered_items = self.items
            .iter()
            .filter(|item| item.name.to_lowercase().contains(&self.filter_text.to_lowercase()))
            .cloned()
            .collect();
        if self.state.selected().is_some() && self.state.selected().unwrap() >= self.filtered_items.len() {
            self.state.select(if self.filtered_items.is_empty() { None } else { Some(0) });
        } else if self.filtered_items.is_empty() {
            self.state.select(None);
        }
    }

    
    pub fn add_char_to_filter(&mut self, ch: char) {
        self.filter_text.push(ch);
        self.update_filtered_items();
    }

    pub fn remove_char_from_filter(&mut self) {
        self.filter_text.pop();
        self.update_filtered_items();
    }

    
    pub fn next(&mut self) {
        if self.filtered_items.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.filtered_items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.filtered_items.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.filtered_items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }

    
    pub fn show_add_popup(&mut self) { 
        self.show_popup = true;
        self.popup_input_focused = 0; 
        self.is_editing = false; 
        self.editing_command_index = None; 
    }

    
    pub fn hide_add_popup(&mut self) { 
        self.show_popup = false;
        self.popup_input_name.clear();
        self.popup_input_desc.clear();
        self.popup_input_command.clear();
        self.popup_input_focused = 0; 
        self.exit_edit_mode(); 
    }
    
    pub fn save_command(&mut self) {
        if self.is_editing { 
            if let Some(index) = self.editing_command_index {
                if let Some(command_to_edit) = self.items.get_mut(index) { 
                    command_to_edit.name = self.popup_input_name.clone();
                    command_to_edit.desc = self.popup_input_desc.clone();
                    command_to_edit.command = self.popup_input_command.clone();
                }
            }
        } else { 
            let new_command = BashCmd {
                name: self.popup_input_name.clone(),
                desc: self.popup_input_desc.clone(),
                command: self.popup_input_command.clone(),
                index: (self.items.len() + 1) as u8,
                category: 1,
                favourite: false,
            };
            self.items.push(new_command);
        }
        self.update_filtered_items();
        self.hide_add_popup(); 
    }

    
    pub fn enter_edit_mode(&mut self, index: usize) {
        self.is_editing = true;
        self.editing_command_index = Some(index);

        
        if let Some(command_to_edit) = self.filtered_items.get(index) {
            self.popup_input_name = command_to_edit.name.clone();
            self.popup_input_desc = command_to_edit.desc.clone();
            self.popup_input_command = command_to_edit.command.clone();
        }
        self.show_popup = true; 
    }

    pub fn exit_edit_mode(&mut self) {
        self.is_editing = false;
        self.editing_command_index = None;
    }
    
    pub fn add_char_to_popup_input_name(&mut self, ch: char) {
        self.popup_input_name.push(ch);
    }
    pub fn remove_char_from_popup_input_name(&mut self) {
        self.popup_input_name.pop();
    }
    
    pub fn add_char_to_popup_input_desc(&mut self, ch: char) { 
        self.popup_input_desc.push(ch);
    }
    pub fn remove_char_from_popup_input_desc(&mut self) { 
        self.popup_input_desc.pop();
    }
    pub fn add_char_to_popup_input_command(&mut self, ch: char) { 
        self.popup_input_command.push(ch);
    }
    pub fn remove_char_from_popup_input_command(&mut self) { 
        self.popup_input_command.pop();
    }
    pub fn next_popup_input_focus(&mut self) {
        self.popup_input_focused = (self.popup_input_focused + 1) % 3; 
    }
}