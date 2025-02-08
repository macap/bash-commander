use ratatui::{
    backend::CrosstermBackend,
    Terminal,
    widgets::{List, ListItem, Block, Borders, ListState, Paragraph, BorderType, Clear},
    layout::{Layout, Constraint, Direction, Rect, Alignment},
    style::{Style, Color, Modifier},
    text::{Span, Text, Line},
};
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

use std::os::unix::process::CommandExt;

#[macro_use] extern crate run_shell;

// Definicja struktury BashCmd z implementacją Clone i Copy
#[derive(Clone)]
struct BashCmd {
    name: String,
    desc: String,
    command: String,
    index: u8,
    category: u8,
    favourite: bool,
}

// Struktura aplikacji do przechowywania stanu listy, inputu i popupu
// Struktura aplikacji App - ZMODYFIKOWANA - DODANO popup_input_focused
struct App {
    items: Vec<BashCmd>,
    filtered_items: Vec<BashCmd>,
    state: ListState,
    filter_text: String,
    show_popup: bool,
    popup_input_name: String,
    popup_input_desc: String,
    popup_input_command: String,
    popup_input_focused: u8, // Dodano pole stanu - aktywny input w popupie (0-Nazwa, 1-Opis, 2-Komenda)
    is_editing: bool,               // Dodano stan - czy jesteśmy w trybie edycji
    editing_command_index: Option<usize>, // Dodano stan - indeks edytowanej komendy (w filtered_items)
    selected_command_to_execute: Option<String>, // NOWE POLE - Przechowuje wybraną komendę do wykonania
}

fn get_commands_file_path() -> PathBuf {
    let mut path = PathBuf::new();
    if let Some(home_dir) = dirs::home_dir() {
        path.push(home_dir);
        path.push(".config");
        path.push("bash_command_app");
        fs::create_dir_all(&path).unwrap_or_else(|_| {});
        path.push("commands.txt"); // Zmień nazwę pliku na commands.txt
    }
    path
}


fn save_commands_to_file(app: &App) -> io::Result<()> {
    let path = get_commands_file_path();
    let mut file = fs::File::create(path)?; // Utwórz plik
    for command in &app.items { // Iteruj po komendach
        writeln!(file, "{name}|{desc}|{command}", // Zapisz linię w formacie: name|desc|command
                 name = command.name,
                 desc = command.desc,
                 command = command.command)?;
    }
    Ok(())
}

fn load_commands_from_file() -> io::Result<Vec<BashCmd>> {
    let path = get_commands_file_path();
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = fs::File::open(path)?; // Otwórz plik do odczytu
    let reader = BufReader::new(file); // Użyj BufReader do efektywnego czytania linii

    let mut commands = Vec::new();
    for line_result in reader.lines() { // Iteruj po liniach pliku
        let line = line_result?; // Obsłuż błąd odczytu linii
        let parts: Vec<&str> = line.splitn(3, '|').collect(); // Podziel linię na 3 części (name|desc|command) używając '|' jako separatora, max 3 podziały
        if parts.len() == 3 { // Upewnij się, że linia ma 3 części
            let name = parts[0].trim().to_string(); // Pobierz nazwę, trimuj białe znaki i konwertuj na String
            let desc = parts[1].trim().to_string(); // Pobierz opis, trimuj i konwertuj
            let command_text = parts[2].trim().to_string(); // Pobierz komendę, trimuj i konwertuj

            // Utwórz BashCmd z sparsowanych danych (index, category, favourite - domyślne wartości)
            let bash_cmd = BashCmd {
                name,
                desc,
                command: command_text,
                index: (commands.len() + 1) as u8, // Ustaw index na podstawie pozycji na liście
                category: 1, // Domyślna kategoria
                favourite: false, // Domyślnie nieulubiona
            };
            commands.push(bash_cmd); // Dodaj komendę do wektora
        }
        // Jeśli linia nie ma 3 części (np. pusta linia lub nieprawidłowy format), ignoruj ją
    }
    Ok(commands)
}



impl App {
    fn new() -> App {
         let initial_items = vec![
            BashCmd { name: "ls".to_string(), desc: "List files".to_string(), command: "ls -l".to_string(), index: 1, category: 1, favourite: false },
         ];

        let loaded_commands = load_commands_from_file().unwrap_or_else(|_| Vec::new()); // Wczytaj komendy z pliku, w przypadku błędu użyj pustego wektora
        let items_to_use = if !loaded_commands.is_empty() { // Jeśli wczytano komendy z pliku, użyj ich
            loaded_commands
        } else { // W przeciwnym razie użyj domyślnych komend
            initial_items
        };

        App {
            items: items_to_use.clone(),
            filtered_items: items_to_use.clone(),
            state: ListState::default(),
            filter_text: String::new(),
            show_popup: false, // Inicjalizacja popupu jako ukrytego
            popup_input_name: String::new(),    // Inicjalizacja inputu Nazwa
            popup_input_desc: String::new(),    // Inicjalizacja inputu Opis
            popup_input_command: String::new(), // Inicjalizacja inputu Komenda
            popup_input_focused: 0, // Inicjalizacja aktywnego inputu na "Nazwa" (indeks 0)
            is_editing: false,                // Inicjalizacja - nie jesteśmy w trybie edycji
            editing_command_index: None,         // Inicjalizacja - brak edytowanej komendy
            selected_command_to_execute: None, // Inicjalizacja na None - brak wybranej komendy na początku
            
        }
    }

    // Metoda do filtrowania listy BashCmd (bez zmian)
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

    // Metody do obsługi filtra (bez zmian)
    fn add_char_to_filter(&mut self, ch: char) {
        self.filter_text.push(ch);
        self.update_filtered_items();
    }

    fn remove_char_from_filter(&mut self) {
        self.filter_text.pop();
        self.update_filtered_items();
    }

    // Metody nawigacji po liście (bez zmian)
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

    // Metoda do wyświetlania popupu dodawania - ustawia flagę
    pub fn show_add_popup(&mut self) { // Metody popupu bez zmian
        self.show_popup = true;
        self.popup_input_focused = 0; // Po pokazaniu popupu, fokus na input "Nazwa"
        self.is_editing = false; // Reset - domyślnie dodawanie, nie edycja
        self.editing_command_index = None; // Reset - brak edytowanej komendy
    }

    // Metoda do ukrywania popupu dodawania - ustawia flagę
    pub fn hide_add_popup(&mut self) { // Metody popupu hide - ZMODYFIKOWANE - RESET FOCUS
        self.show_popup = false;
        self.popup_input_name.clear();
        self.popup_input_desc.clear();
        self.popup_input_command.clear();
        self.popup_input_focused = 0; // Po ukryciu popupu, reset fokus na "Nazwa" (opcjonalne, ale logiczne)
        self.exit_edit_mode(); // Wywołaj exit_edit_mode przy zamykaniu popupu
    }
    // Metoda save_command - ZMODYFIKOWANA - OBSŁUGA EDYCJI I DODAWANIA
    fn save_command(&mut self) {
        if self.is_editing { // TRYB EDYCJI
            if let Some(index) = self.editing_command_index {
                if let Some(command_to_edit) = self.items.get_mut(index) { // Pobierz mutowalną referencję do komendy w `items`
                    command_to_edit.name = self.popup_input_name.clone();
                    command_to_edit.desc = self.popup_input_desc.clone();
                    command_to_edit.command = self.popup_input_command.clone();
                }
            }
        } else { // TRYB DODAWANIA (stary kod add_new_command)
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
        self.hide_add_popup(); // hide_add_popup teraz wywołuje exit_edit_mode()
    }

    // NOWE metody enter_edit_mode i exit_edit_mode
    pub fn enter_edit_mode(&mut self, index: usize) {
        self.is_editing = true;
        self.editing_command_index = Some(index);

        // Wypełnij pola inputu danymi edytowanej komendy
        if let Some(command_to_edit) = self.filtered_items.get(index) {
            self.popup_input_name = command_to_edit.name.clone();
            self.popup_input_desc = command_to_edit.desc.clone();
            self.popup_input_command = command_to_edit.command.clone();
        }
        self.show_popup = true; // Otwórz popup
    }

    pub fn exit_edit_mode(&mut self) {
        self.is_editing = false;
        self.editing_command_index = None;
    }
    // NOWE METODY obsługi inputów w popupie - dla Nazwy
    pub fn add_char_to_popup_input_name(&mut self, ch: char) {
        self.popup_input_name.push(ch);
    }
    pub fn remove_char_from_popup_input_name(&mut self) {
        self.popup_input_name.pop();
    }
    // Metody dla Opisu i Komendy dodamy w kolejnych krokach
    pub fn add_char_to_popup_input_desc(&mut self, ch: char) { // Dodane (puste na razie)
        self.popup_input_desc.push(ch);
    }
    pub fn remove_char_from_popup_input_desc(&mut self) { // Dodane (puste na razie)
        self.popup_input_desc.pop();
    }
    pub fn add_char_to_popup_input_command(&mut self, ch: char) { // Dodane (puste na razie)
        self.popup_input_command.push(ch);
    }
    pub fn remove_char_from_popup_input_command(&mut self) { // Dodane (puste na razie)
        self.popup_input_command.pop();
    }
    pub fn next_popup_input_focus(&mut self) {
        self.popup_input_focused = (self.popup_input_focused + 1) % 3; // Przełącz na następny input (0->1->2->0)
    }
}
// Funkcja draw_command_details - NOWA FUNKCJA - Rysowanie szczegółów komendy
fn draw_command_details(f: &mut ratatui::Frame, command: Option<&BashCmd>, area: Rect) {
    let block = Block::default().title("Szczegóły Komendy").borders(Borders::ALL).border_type(BorderType::Rounded);
    f.render_widget(Block::default().borders(Borders::ALL).title("Szczegóły Komendy").border_type(BorderType::Rounded), area);

    let details_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3), // Nazwa
            Constraint::Length(3), // Opis
            Constraint::Length(3), // Komenda
            Constraint::Length(3), // Kategoria (na razie puste, możemy dodać kategorię do BashCmd później)
            Constraint::Min(0),    // Reszta przestrzeni
        ].as_ref())
        .split(area);

    if let Some(cmd) = command { // Jeśli komenda jest wybrana (Some(&BashCmd))
        let name_paragraph = Paragraph::new(Text::from(Line::from(vec![Span::styled("Nazwa: ", Style::default().add_modifier(Modifier::BOLD)), Span::raw(cmd.name.as_str())])))
            .block(Block::default().borders(Borders::BOTTOM));
        f.render_widget(name_paragraph, details_layout[0]);

        let desc_paragraph = Paragraph::new(Text::from(Line::from(vec![Span::styled("Opis: ", Style::default().add_modifier(Modifier::BOLD)), Span::raw(cmd.desc.as_str())])))
            .block(Block::default().borders(Borders::BOTTOM));
        f.render_widget(desc_paragraph, details_layout[1]);

        let command_paragraph = Paragraph::new(Text::from(Line::from(vec![Span::styled("Komenda: ", Style::default().add_modifier(Modifier::BOLD)), Span::raw(cmd.command.as_str())])))
            .block(Block::default().borders(Borders::BOTTOM));
        f.render_widget(command_paragraph, details_layout[2]);

        let category_paragraph = Paragraph::new(Text::from(Line::from(vec![Span::styled("Kategoria: ", Style::default().add_modifier(Modifier::BOLD)), Span::raw(cmd.category.to_string())]))) // Na razie wyświetlamy kategorię
            .block(Block::default());
        f.render_widget(category_paragraph, details_layout[3]);

    } else { // Jeśli brak wybranej komendy (None)
        let placeholder_text = Text::from("Wybierz komendę z listy,\naby zobaczyć szczegóły.");
        let placeholder_paragraph = Paragraph::new(placeholder_text)
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::NONE).title("Szczegóły Komendy").border_type(BorderType::Rounded));
        f.render_widget(placeholder_paragraph, area); // Rysuj placeholder na całej dostępnej przestrzeni
    }
}

// Funkcja do rysowania PUSTEGO popupu dodawania
fn draw_add_popup(f: &mut ratatui::Frame, app: &App) {
    let popup_title = if app.is_editing { "Edytuj Komendę" } else { "Dodaj Komendę" };
    
    let block = Block::default().title(popup_title).borders(Borders::ALL).border_type(BorderType::Rounded);
    let popup_area = left_aligned_rect(60, 40, f.size());
    f.render_widget(Clear, popup_area);
    f.render_widget(block, popup_area);

    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3), // Nazwa - INPUT
            Constraint::Length(3), // Opis  - INPUT
            Constraint::Length(3), // Komenda - INPUT
            Constraint::Min(0),    // Reszta przestrzeni
        ].as_ref())
        .split(popup_area);

      // Styl aktywnego inputu - NOWY STYL
    let focused_style = Style::default().fg(Color::LightGreen).add_modifier(Modifier::BOLD);
    let normal_style = Style::default();

    // INPUT - Nazwa -  STYL ZALEŻNY OD FOKUSU
    let name_block = Block::default().borders(Borders::ALL).title("Nazwa")
        .border_style(if app.popup_input_focused == 0 { focused_style } else { normal_style }); // Aktywny styl dla inputu Nazwa, gdy fokus == 0
    let name_paragraph = Paragraph::new(Text::from(app.popup_input_name.as_str()))
        .block(name_block);
    f.render_widget(name_paragraph, popup_layout[0]);

    // INPUT - Opis - STYL ZALEŻNY OD FOKUSU
    let desc_block = Block::default().borders(Borders::ALL).title("Opis")
        .border_style(if app.popup_input_focused == 1 { focused_style } else { normal_style }); // Aktywny styl dla inputu Opis, gdy fokus == 1
    let desc_paragraph = Paragraph::new(Text::from(app.popup_input_desc.as_str()))
        .block(desc_block);
    f.render_widget(desc_paragraph, popup_layout[1]);

    // INPUT - Komenda - STYL ZALEŻNY OD FOKUSU
    let command_block = Block::default().borders(Borders::ALL).title("Komenda")
        .border_style(if app.popup_input_focused == 2 { focused_style } else { normal_style }); // Aktywny styl dla inputu Komenda, gdy fokus == 2
    let command_paragraph = Paragraph::new(Text::from(app.popup_input_command.as_str()))
        .block(command_block);
    f.render_widget(command_paragraph, popup_layout[2]);
}

// Funkcja pomocnicza do centrowania prostokąta na ekranie (bez zmian)
fn left_aligned_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_width = r.width * percent_x / 100;
    let popup_height = r.height * percent_y / 100;

    Rect {
        x: r.left() + 2, // Margines od lewej krawędzi (opcjonalnie)
        y: r.top() + 2,  // Margines od górnej krawędzi (opcjonalnie)
        width: popup_width,
        height: popup_height,
    }
}


fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<Option<String>> {
    loop {
        terminal.draw(|f| {
            let main_layout = Layout::default() // ZMODYFIKOWANO - UŻYJ POZIOMEGO UKŁADU DLA LISTY I SZCZEGÓŁÓW
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(3),      // Filtr
                        Constraint::Percentage(90), // Lista i Szczegóły - TERAZ POZIOMO PODZIELONE
                        Constraint::Length(2),      // Pomoc
                    ].as_ref()
                )
                .split(f.size());

            let content_layout = Layout::default() // NOWY POZIOMY UKŁAD DLA LISTY I SZCZEGÓŁÓW
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(50), // Lewa strona - Lista
                    Constraint::Percentage(50), // Prawa strona - Szczegóły
                ].as_ref())
                .split(main_layout[1]); // Użyj drugiego chunk'a z głównego układu (obszar listy i szczegółów)


            let input_paragraph = Paragraph::new(Text::from(app.filter_text.as_str()))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title("Filtr")
                    );
            f.render_widget(input_paragraph, main_layout[0]);


            // Tworzenie elementów listy (bez zmian)
            let items: Vec<ListItem> = app.filtered_items
                .iter()
                .map(|item| {
                    ListItem::new(Span::styled(item.name.clone(), Style::default().fg(Color::Gray)))
                })
                .collect();

            let list = List::new(items)
                .block(Block::default().title("Lista Komend Bash").borders(Borders::ALL))
                .highlight_style(Style::default().add_modifier(Modifier::BOLD).bg(Color::LightGreen))
                .highlight_symbol("> ");

            f.render_stateful_widget(list, content_layout[0], &mut app.state);

             // SZCZEGÓŁY KOMENDY - RYSOWANE W PRAWEJ CZĘŚCI content_layout
            let selected_command = match app.state.selected() { // Pobierz wybrana komendę (Option<&BashCmd>)
                Some(index) => app.filtered_items.get(index),
                None => None,
            };
            draw_command_details(f, selected_command, content_layout[1]); // Rysuj szczegóły w prawym chunk


            // Rysowanie popupu, jeśli show_popup jest true - WARUNKOWE RYSOWANIE
            if app.show_popup {
                draw_add_popup(f, &app);
            }

                let help_text = Text::from(Line::from(vec![ // Używamy Line do łatwiejszego formatowania w jednej linii
                Span::styled("ESC / Ctrl+Q", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(": Wyjście"),
                Span::raw(" | "), // Separator
                Span::styled("Ctrl+A", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(": Dodaj Komendę"),
                Span::raw(" | "), // Separator
                Span::styled("Ctrl+E", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(": Edytuj Komendę"),
                // Możesz dodać więcej skrótów tutaj, np. do nawigacji, ulubionych, etc.
            ]));
            let help_paragraph = Paragraph::new(help_text)
                .style(Style::default().fg(Color::LightCyan)) // Styl szarego tekstu
                .alignment(Alignment::Center) // Wycentrowanie tekstu w belce
                .block(Block::default().borders(Borders::TOP)); // Górna linia oddzielająca belkę

            f.render_widget(help_paragraph, main_layout[2]); // Rysujemy belkę w ostatnim chunku (chunks[2])
   
        })?;

        if let Event::Key(key) = crossterm::event::read()? {
            // Obsługa klawiszy, gdy popup jest UKRYTY (standardowa obsługa aplikacji) - ZMODYFIKOWANE
            if !app.show_popup { // Sprawdzamy, czy popup NIE jest widoczny
                match key.code {
                    KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        save_commands_to_file(&app)?; 
                        return Ok(None)
                    }, // **Ctrl + q TERAZ WYJŚCIE!**
                    KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => app.show_add_popup(), // **Ctrl + a TERAZ POPUP!**
                    KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => { // **Ctrl+e - EDYCJA KOMENDY**
                        if let Some(selected_index) = app.state.selected() { // Sprawdź czy komenda jest podświetlona
                            app.enter_edit_mode(selected_index); // Przejdź do trybu edycji dla podświetlonej komendy
                        }
                    },
                    KeyCode::Enter => {
                       if let Some(selected_index) = app.state.selected() {
                            if let Some(selected_command) = app.filtered_items.get(selected_index) {
                                // Zapisz TEKST komendy w app.selected_command_to_execute
                                app.selected_command_to_execute = Some(selected_command.command.clone()); // Zapisz клонированный tekst komendy
                                save_commands_to_file(&app)?; // Opcjonalne zapisywanie po Enter
                                return Ok(app.selected_command_to_execute.clone()); // Zwróć Ok(Some(String)) - komenda wybrana
                            } else {
                                app.selected_command_to_execute = None;
                                return Ok(None); // Teoretycznie nie powinno się zdarzyć, ale na wszelki wypadek
                            }
                        } else {
                            app.selected_command_to_execute = None; // Brak wybranej komendy
                            // return Ok(None); // Zwróć Ok(None) - brak komendy do wykonania
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
            } else { // Obsługa klawiszy, gdy popup jest WIDOCZNY - DODANA OBSŁUGA HIDE POPUP
               match key.code {
                    KeyCode::Esc => app.hide_add_popup(),
                    KeyCode::Enter => app.save_command(),
                    KeyCode::Tab => app.next_popup_input_focus(), // Obsługa Tab - przełączanie fokusu inputu
                    KeyCode::Char(ch) => { // Wprowadzanie tekstu - TERAZ KIEROWANE DO AKTYWNEGO INPUTU
                        match app.popup_input_focused {
                            0 => app.add_char_to_popup_input_name(ch), // Input "Nazwa"
                            1 => app.add_char_to_popup_input_desc(ch), // Input "Opis"
                            2 => app.add_char_to_popup_input_command(ch), // Input "Komenda"
                            _ => {} // Domyślny przypadek (nie powinien się zdarzyć)
                        }
                    }
                    KeyCode::Backspace => { // Backspace - TERAZ KIEROWANY DO AKTYWNEGO INPUTU
                        match app.popup_input_focused {
                            0 => app.remove_char_from_popup_input_name(), // Backspace dla inputu "Nazwa"
                            1 => app.remove_char_from_popup_input_desc(), // Backspace dla inputu "Opis"
                            2 => app.remove_char_from_popup_input_command(), // Backspace dla inputu "Komenda"
                            _ => {} // Domyślny przypadek (nie powinien się zdarzyć)
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = App::new();
    let res = run_app(&mut terminal, app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    let selected_command_string_option = match res { // Dopasuj Result<Option<String>>
        Ok(command_string_option) => command_string_option, // Pobierz Option<String> z OK
        Err(err) => {
            println!("{:?}", err);
            None // W przypadku błędu, ustaw None - brak komendy do wykonania
        }
    };

    

    if let Some(command_to_execute) = selected_command_string_option  { // Sprawdź, czy komenda została wybrana

        let parts: Vec<&str> = command_to_execute.split_whitespace().collect();

        if let Some(command_name) = parts.get(0) {
            let mut command = Command::new(command_name);

            for arg in parts.iter().skip(1) {
                command.arg(arg);
            }
            command.exec();
        } else {
            eprintln!("Pusta komenda do wykonania.");
        }
    }

    Ok(())
}