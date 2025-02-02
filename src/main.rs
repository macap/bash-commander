extern crate run_shell;
use run_shell::*;
use crossterm::event::{self, Event};
use ratatui::{text::Text, Frame};

struct BashCmd<'a> {
    name: &'a str,
    desc: &'a str,
    command: &'a str,
    index: u8,
    category: u8,
    favourite: bool
}

const commands: [BashCmd; 2] = [
    BashCmd { name: "list dir", desc: "contents of current dir", command: "ls", index: 0, category: 0, favourite: false },
    BashCmd { name: "list dir", desc: "contents of current dir", command: "ls", index: 0, category: 0, favourite: false }
];

fn main() {
    let mut terminal = ratatui::init();
    loop {
        terminal.draw(draw).expect("failed to draw frame");
        if matches!(event::read().expect("failed to read event"), Event::Key(_)) {
            break;
        }
    }
    ratatui::restore();
}

fn draw(frame: &mut Frame) {
    let res = cmd!("ls").stdout_utf8().unwrap();
    let text = Text::raw(res);
    frame.render_widget(text, frame.area());
}
