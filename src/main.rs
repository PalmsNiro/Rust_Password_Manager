mod db;

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
use std::fmt::format;
use tui::backend::{Backend, CrosstermBackend};
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::Span;
use tui::widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph};
use tui::{Frame, Terminal};

use crate::db::Database;

//Passphrase wird beim ersten mal fragen festgelegt und gespeichert mit der erstellung der Datenbank
//Um diesen zu resetten muss die Datenbank gelöscht werden

const APP_KEYS_DESC: &str = r#"
L:           List
U:           On list, It's copy the Username
P:           On list, It's copy the Password
D:           On list, It's Delete
E:           On list, It's Edit
S:           Search
I:           Insert new Password
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
    Delete,
}

#[derive(Clone)]
struct Password {
    id: usize,
    title: String,
    username: String,
    password: String,
}
impl Password {
    pub fn new(title: String, username: String, password: String) -> Password {
        Password {
            id: 0,
            title,
            username,
            password,
        }
    }

    pub fn new_with_id(id: usize, title: String, username: String, password: String) -> Password {
        Password {
            id,
            title,
            username,
            password,
        }
    }
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
    db: Database,
}
impl PassMng {
    pub fn new(key: String) -> PassMng {
        let db = match Database::new(key) {
            Ok(db) => db,
            Err(e) => {
                println!("no uhuh");
                println!("{}", e.to_string());
                std::process::exit(1);
            }
        };
        let passwords = db.load();
        PassMng {
            mode: InputMode::Normal,
            list_state: ListState::default(),
            passwords,
            search_txt: String::new(),
            search_list: vec![],
            new_title: String::new(),
            new_username: String::new(),
            new_password: String::new(),
            db,
        }
    }

    pub fn change_mode(&mut self, mode: InputMode) {
        self.mode = mode;
    }

    pub fn clear_fields(&mut self) {
        self.new_title.clear();
        self.new_username.clear();
        self.new_password.clear();
    }

    pub fn insert(&mut self) {
        let password = Password {
            id: 32,
            title: self.new_title.to_owned(),
            username: self.new_username.to_owned(),
            password: self.new_password.to_owned(),
        };
        self.db.insert_password(&password);
        self.passwords.push(password);
        self.clear_fields();
        self.change_mode(InputMode::Normal);
    }

    pub fn search(&mut self) {
        self.search_list = self
            .passwords
            .clone()
            .into_iter()
            .filter(|item| item.title.starts_with(&self.search_txt.to_owned()))
            .collect();
    }

    pub fn edit(&mut self) {}

    pub fn copy_username(&mut self) {
        if let Some(index) = self.list_state.selected() {
            let username = &self.passwords[index].username;
            let mut clipboard = Clipboard::new().unwrap();
            clipboard.set_text(username).unwrap();
        };
    }

    pub fn copy_password(&mut self) {
        if let Some(index) = self.list_state.selected() {
            let password = &self.passwords[index].password;
            let mut clipboard = Clipboard::new().unwrap();
            clipboard.set_text(password).unwrap();
        };
    }

    pub fn move_up(&mut self) {
        let selected = match self.list_state.selected() {
            Some(v) => {
                if v == 0 {
                    Some(v)
                } else {
                    Some(v - 1)
                }
            }
            None => Some(0),
        };
        self.list_state.select(selected);
    }

    pub fn move_down(&mut self) {
        let selected = match self.list_state.selected() {
            Some(v) => {
                if v == self.passwords.len() - 1 {
                    Some(v)
                } else {
                    Some(v + 1)
                }
            }
            None => Some(0),
        };
        self.list_state.select(selected);
    }

    pub fn delete_password(&mut self) {
        if let Some(index) = self.list_state.selected() {
            let id = self.passwords[index].id;
            self.db.delete_pw(id);
            self.passwords.remove(index);
            self.list_state.select(None);
        };
    }

}

fn main() -> Result<(), Box<dyn Error>> {
    let passphrase = rpassword::prompt_password("Passwort zum entsperren: ").unwrap();
    let mut state = PassMng::new(passphrase);

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
                        state.change_mode(InputMode::Search);
                    }
                    KeyCode::Char('l') => {
                        state.change_mode(InputMode::List);
                    }
                    KeyCode::Char('i') => {
                        state.change_mode(InputMode::Title);
                    }
                    _ => {}
                },

                InputMode::Title => match key.code {
                    KeyCode::Esc => {
                        state.clear_fields();
                        state.change_mode(InputMode::Normal);
                    }
                    KeyCode::Char(c) => {
                        state.new_title.push(c);
                    }
                    KeyCode::Backspace => {
                        state.new_title.pop();
                    }

                    KeyCode::Tab => {
                        state.change_mode(InputMode::Username);
                    }
                    _ => {}
                },
                InputMode::Username => match key.code {
                    KeyCode::Esc => {
                        state.clear_fields();
                        state.change_mode(InputMode::Normal);
                    }
                    KeyCode::Char(c) => {
                        state.new_username.push(c);
                    }
                    KeyCode::Backspace => {
                        state.new_username.pop();
                    }
                    KeyCode::Tab => {
                        state.change_mode(InputMode::Password);
                    }
                    KeyCode::BackTab => {
                        state.change_mode(InputMode::Title);
                    }
                    _ => {}
                },
                InputMode::Password => match key.code {
                    KeyCode::Esc => {
                        state.clear_fields();
                        state.change_mode(InputMode::Normal);
                    }
                    KeyCode::Char(c) => {
                        state.new_password.push(c);
                    }
                    KeyCode::Backspace => {
                        state.new_password.pop();
                    }
                    KeyCode::Tab => {
                        state.change_mode(InputMode::Submit);
                    }
                    KeyCode::BackTab => {
                        state.change_mode(InputMode::Username);
                    }
                    _ => {}
                },
                InputMode::Submit => match key.code {
                    KeyCode::Esc => {
                        state.clear_fields();
                        state.change_mode(InputMode::Normal);
                    }
                    KeyCode::BackTab => {
                        state.change_mode(InputMode::Password);
                    }
                    KeyCode::Enter => {
                        state.insert();
                    }
                    _ => {}
                },
                InputMode::Search => match key.code {
                    KeyCode::Esc => {
                        state.change_mode(InputMode::Normal);
                    }
                    KeyCode::Char(c) => {
                        state.search_txt.push(c);
                        state.search();
                    }
                    KeyCode::Backspace => {
                        state.search_txt.pop();
                        state.search();
                    }
                    KeyCode::Down => {
                        state.change_mode(InputMode::List);
                    }
                    KeyCode::Tab => state.change_mode(InputMode::List),
                    KeyCode::BackTab => state.change_mode(InputMode::List),
                    _ => {}
                },
                InputMode::List => match key.code {
                    KeyCode::Esc => {
                        state.change_mode(InputMode::Normal);
                    }
                    KeyCode::BackTab => state.change_mode(InputMode::Search),
                    KeyCode::Up => {
                        state.move_up();
                    }
                    KeyCode::Down => {
                        state.move_down();
                    }
                    KeyCode::Char('u') => {
                        //Copy Username
                        state.copy_username();
                    }
                    KeyCode::Char('p') => {
                        //Copy Username
                        state.copy_password();
                    }
                    KeyCode::Char('e') => {
                        state.edit();
                    }
                    KeyCode::Char('d') => {
                        state.change_mode(InputMode::Delete);
                    }
                    _ => {}
                },
                InputMode::Delete => match key.code {
                    KeyCode::Char('y') => {
                        state.delete_password();
                        state.change_mode(InputMode::List);
                    }
                    KeyCode::Char('n') => todo!(),
                    _ => {}
                },
            }
        };
    }
}

//fertig
fn ui<B: Backend>(f: &mut Frame<B>, state: &mut PassMng) {
    let parent_chunk = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(f.size());

    //Linker Block für Input und Beschreibung
    let new_section_block = Block::default()
        .title("New Password")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    f.render_widget(new_section_block, parent_chunk[0]);
    new_section(f, state, parent_chunk[0]);

    //Rechter Block für die Passwort Liste
    let list_section_block = Block::default()
        .title("List of Passwords")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    f.render_widget(list_section_block, parent_chunk[1]);
    list_section(f, state, parent_chunk[1]);

    delete_popup(f, state);
}

//fertig
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

//fertig
fn list_section<B: Backend>(f: &mut Frame<B>, state: &mut PassMng, area: Rect) {
    let list_to_show = if state.search_list.is_empty() {
        state.passwords.to_owned()
    } else {
        state.search_list.to_owned()
    };
    let items: Vec<ListItem> = list_to_show
        .into_iter()
        .map(|item| match state.mode {
            InputMode::List => ListItem::new(format!(
                "{}: {} - {}",
                item.title.to_owned(),
                item.username.to_owned(),
                item.password.to_owned()
            )),
            _ => ListItem::new(Span::from(item.title)),
        })
        .collect();

    let list_chunks = Layout::default()
        .margin(2)
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)].as_ref())
        .split(area);

    let search_input = Paragraph::new(state.search_txt.to_owned())
        .block(
            Block::default()
                .title("Search")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(match state.mode {
            InputMode::Search => Style::default().fg(Color::Yellow),
            _ => Style::default(),
        });
    f.render_widget(search_input, list_chunks[0]);

    let list = List::new(items)
        .block(Block::default())
        .highlight_symbol("->")
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );
    f.render_stateful_widget(list, list_chunks[1], &mut state.list_state);
}

fn delete_popup<B: Backend>(f: &mut Frame<B>, state: &mut PassMng) {
    if let InputMode::Delete = state.mode {
        let block = Block::default()
            .title("DELETE")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);
        let area = centered_rect(60, 25, f.size());
        f.render_widget(Clear, area); //this clears out the background
        f.render_widget(block, area);

        let chunk = Layout::default()
            .margin(2)
            .constraints(
                [
                    Constraint::Length(2),
                    Constraint::Length(2),
                ].as_ref()
            )
            .split(area);

        let text = Paragraph::new("Are you sure?")
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center);
        f.render_widget(text, chunk[0]);

        let keys_desc = Paragraph::new("Press (Y) for Yes and (N) for No")
            .alignment(Alignment::Center);
        f.render_widget(keys_desc, chunk[1]);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
                .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
                .as_ref(),
        )
        .split(popup_layout[1])[1]
}