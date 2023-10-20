// mod db;

// use crate::db::Database;
use arboard::Clipboard;
use crossterm::event::Event::Key;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture, KeyCode};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{event, execute};
// use rusqlite::ErrorCode;
use std::error::Error;
use tui::backend::{Backend, CrosstermBackend};
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::Span;
use tui::widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph};
use tui::{Frame, Terminal};



//Aufgehört bei Minute 25:45

const APP_KEYS_DESC: &str = r#"
L:           List
U:           On list, It's copy the Username
P:           On list, It's copy the Password
D:           On list, It's Delete
E:           On list, It's Edit
S:           Search
Insert Btn:  Insert new Password
Tab:         Go to next field
Shift+Tab:   Go to previous filed
Esc:         Exit insert mode
"#;

enum InputMode {
    Normal,
    Title,
    Username,
    Password,
    Submit,
    Search,
    List,
}

struct Password {
    title: String,
    username: String,
    üassword: String,
}

struct PassMng {
    mode: InputMode,
    list_state: ListState,
    passwords: Vec<Password>,
    search_txt: String,
    search_list: Vec<Password>,
    new_title: String,
    new_username: String,
    new_password: String,
}
impl PassMng {
    pub fn new() -> PassMng {
        PassMng {
            mode: InputMode::Normal,
            list_state: ListState::default(),
            passwords: vec![],
            search_txt: String::new(),
            search_list: vec![],
            new_title: String::new(),
            new_username: String::new(),
            new_password: String::new(),
        }
    }

    pub fn chagen_mode(&mut self, mode: InputMode) {
        self.mode = mode;
    }

    pub fn clear_field(&mut self) {
        self.new_title.clear();
        self.new_username.clear();
        self.new_password.clear();
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut state = PassMng::new();

    enable_raw_mode()?;
    execute!(std::io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(std::io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal, &mut state);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
    )?;

    if let Err(e) = result {
        println!("{}", e.to_string())
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    state: &mut PassMng,
) -> Result<(), std::io::Error> {
    loop {
        terminal.draw(|f| ui(f, state))?;

        if let Key(key) = event::read()? {
            match state.mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    KeyCode::Char('s') => {
                        state.chagen_mode(InputMode::Search);
                    }
                    KeyCode::Char('l') => {
                        state.chagen_mode(InputMode::List);
                    }
                    KeyCode::Insert => {
                        state.chagen_mode(InputMode::Title);
                    }
                    _ => {}
                },

                InputMode::Title => match key.code {
                    KeyCode::Esc => {
                        state.clear_field();
                        state.chagen_mode(InputMode::Normal);
                    }
                    KeyCode::Char(c) => {
                        state.new_title.push(c);
                    }
                    KeyCode::Backspace => {
                        state.new_title.pop();
                    }

                    KeyCode::Tab => {
                        state.chagen_mode(InputMode::Username);
                    }
                    _ => {}
                },
                InputMode::Username => match key.code {
                    KeyCode::Esc => {
                        state.clear_field();
                        state.chagen_mode(InputMode::Normal);
                    }
                    KeyCode::Char(c) => {
                        state.new_username.push(c);
                    }
                    KeyCode::Backspace => {
                        state.new_username.pop();
                    }
                    KeyCode::Tab => {
                        state.chagen_mode(InputMode::Password);
                    }
                    KeyCode::BackTab => {
                        state.chagen_mode(InputMode::Title);
                    }
                    _ => {}
                },
                InputMode::Password => match key.code {
                    KeyCode::Esc => {
                        state.clear_field();
                        state.chagen_mode(InputMode::Normal);
                    }
                    KeyCode::Char(c) => {
                        state.new_password.push(c);
                    }
                    KeyCode::Backspace => {
                        state.new_password.pop();
                    }
                    KeyCode::Tab => {
                        state.chagen_mode(InputMode::Submit);
                    }
                    KeyCode::BackTab => {
                        state.chagen_mode(InputMode::Username);
                    }
                    _ => {}
                },
                InputMode::Submit => match key.code {
                    KeyCode::Esc => {
                        state.clear_field();
                        state.chagen_mode(InputMode::Normal);
                    }
                    KeyCode::BackTab => {
                        state.chagen_mode(InputMode::Password);
                    }
                    _ => {}
                },
                InputMode::Search => match key.code {
                    KeyCode::Esc => {
                        state.chagen_mode(InputMode::Normal);
                    }
                    KeyCode::Char(c) => {
                        state.search_txt.push(c);
                    }
                    KeyCode::Backspace => {
                        state.search_txt.pop();
                    }
                    _ => {}
                },
                InputMode::List => match key.code {
                    KeyCode::Esc => {
                        state.chagen_mode(InputMode::Normal);
                    }
                    _ => {}
                },
            }
        };
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, state: &mut PassMng) {
    let parent_chunk = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(f.size());

    let new_section_block = Block::default()
        .title("New Password")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    f.render_widget(new_section_block, parent_chunk[0]);
    new_section(f, state, parent_chunk[0]);

    let list_section_block = Block::default()
        .title("List of Passwords")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    f.render_widget(list_section_block, parent_chunk[1]);
}

fn new_section<B: Backend>(f: &mut Frame<B>, state: &mut PassMng, area: Rect) {
    let new_section_chunk = Layout::default()
        .margin(2)
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Min(4),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(area);

    //laden der Tastenbeschreibung in der UI
    let desc = Paragraph::new(APP_KEYS_DESC);
    f.render_widget(desc, new_section_chunk[0]);

    //Laden des Title Input Feldes
    let title_input = Paragraph::new(state.new_title.to_owned())
        .block(
            Block::default()
                .title("Title")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(match state.mode {
            InputMode::Title => Style::default().fg(Color::Yellow),
            _ => Style::default(),
        });
    f.render_widget(title_input, new_section_chunk[1]);

    //Laden des Username Input Feldes
    let username_input = Paragraph::new(state.new_username.to_owned())
        .block(
            Block::default()
                .title("Username")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(match state.mode {
            InputMode::Username => Style::default().fg(Color::Yellow),
            _ => Style::default(),
        });
    f.render_widget(username_input, new_section_chunk[2]);

    //Laden des Passwort Input Feldes
    let password_input = Paragraph::new(state.new_password.to_owned())
        .block(
            Block::default()
                .title("Password")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(match state.mode {
            InputMode::Password => Style::default().fg(Color::Yellow),
            _ => Style::default(),
        });
    f.render_widget(password_input, new_section_chunk[3]);

    //Laden des Submit buttons
    let submit_btn = Paragraph::new("Submit")
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .title("Submit")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(match state.mode {
            InputMode::Submit => Style::default().fg(Color::Yellow),
            _ => Style::default(),
        });
    f.render_widget(submit_btn, new_section_chunk[4]);
}

fn list_section<B: Backend>(f: &mut Frame<B>, state: &mut PassMng) {}
