use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use model::{common::Task, util::is_completed};
use ratatui::{prelude::Backend, widgets::ListState, Terminal};
use std::{
    error::Error,
    fs::{self, create_dir_all, File},
    io::{BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
};

use crate::ui::render;

#[derive(Debug, Default, PartialEq)]
pub enum CurrentEditing {
    #[default]
    Description,
    Body,
}

#[derive(Debug, Default, PartialEq)]
pub enum CurrentScreen {
    #[default]
    Main,
    Editing,
    Deleting,
    Help,
    Exiting,
}

#[derive(Debug, Default)]
pub struct App {
    pub tasks: Vec<Task>,
    pub current_screen: CurrentScreen,
    pub current_editing: CurrentEditing,
    pub quit: bool,
    pub show_done: bool, // TODO: better data structure
    pub loading: bool,
    pub state: ListState,
    pub throbber_state: throbber_widgets_tui::ThrobberState,
    pub path: PathBuf,
    pub path_bin: PathBuf,
    pub with_json: bool,
    pub buffer: String,
    pub editing: bool,
    pub character_index: usize,
}

impl App {
    pub fn new(path: PathBuf) -> Result<App, Box<dyn Error>> {
        if let Some(parent) = path.parent() {
            if let Err(e) = create_dir_all(parent) {
                eprintln!("Error creating directory {:?}: {}", parent, e);
                std::process::exit(1);
            }
        }

        let mut user_data = File::options()
            .read(true)
            .append(true)
            .create(true)
            .open(&path)?;

        let mut buf = String::new();
        user_data.read_to_string(&mut buf)?;

        let tasks = if buf.is_empty() {
            Vec::new()
        } else {
            buf.lines().map(Task::from_line).collect::<Result<_, _>>()?
        };

        Ok(App {
            tasks,
            state: ListState::default().with_selected(Some(0)),
            path_bin: path,
            ..Default::default()
        })
    }

    pub fn with_json(path: PathBuf) -> Result<App, Box<dyn Error>> {
        let file = Path::new(path.as_path());

        if let Some(parent) = path.parent() {
            if let Err(e) = create_dir_all(parent) {
                eprintln!("Error creating directory {:?}: {}", parent, e);
                std::process::exit(1);
            }
        }

        match !file.exists() {
            true => {
                fs::File::create(path.clone())?;
                Ok(App {
                    tasks: Vec::new(),
                    path: path.to_owned(),
                    with_json: true,
                    ..Default::default()
                })
            }
            false => {
                let buf = Self::read_from_json(path.clone())?;
                Ok(App {
                    tasks: buf,
                    path: path.to_owned(),
                    with_json: true,
                    state: ListState::default().with_selected(Some(0)),
                    ..Default::default()
                })
            }
        }
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> color_eyre::Result<()> {
        let tick_rate = std::time::Duration::from_millis(250);
        let mut last_tick = std::time::Instant::now();
        while !self.quit {
            terminal.draw(|frame| render(self, frame))?;
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| std::time::Duration::from_secs(0));
            if ratatui::crossterm::event::poll(timeout)? {
                self.handle_envents()?;
            }
            if last_tick.elapsed() >= tick_rate {
                self.on_tick();
                last_tick = std::time::Instant::now();
            }
        }
        Ok(())
    }

    fn on_tick(&mut self) {
        self.throbber_state.calc_next();
    }

    fn handle_envents(&mut self) -> color_eyre::Result<()> {
        match event::read()? {
            // it's important to check KeyEventKind::Press to avoid handling key release events
            Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            _ => {}
        }
        Ok(())
    }

    fn on_key_event(&mut self, key: KeyEvent) {
        match self.current_screen {
            CurrentScreen::Main => match (key.modifiers, key.code) {
                (_, KeyCode::Esc | KeyCode::Char('q'))
                | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => {
                    self.current_screen = CurrentScreen::Exiting
                }
                (_, KeyCode::Up) => self.previous(),
                (_, KeyCode::Down) => self.next(),
                (_, KeyCode::Enter) => {
                    // TODO: Check if there are notes to edit. BUG!!
                    self.current_screen = CurrentScreen::Editing;
                    self.editing = true;
                    match self.current_editing {
                        CurrentEditing::Description => self
                            .buffer
                            .push_str(&self.tasks[self.state.selected().unwrap()].description()),
                        CurrentEditing::Body => self
                            .buffer
                            .push_str(&self.tasks[self.state.selected().unwrap()].body()),
                    }
                    self.character_index = self.buffer.chars().count();
                }
                (_, KeyCode::Tab) => {
                    self.change_task_done(self.state.selected().unwrap())
                        .unwrap();
                    self.save_to_file().unwrap();
                }
                (_, KeyCode::Char('n') | KeyCode::Char('N')) => {
                    self.current_screen = CurrentScreen::Editing;
                    self.editing = true;
                    self.buffer.push_str("Type something...");
                    self.character_index = self.buffer.chars().count();
                    self.add_task(Task::from_description(&self.buffer).unwrap())
                        .unwrap_or_default();
                    self.state.select_last();
                }
                (_, KeyCode::Delete) => {
                    self.current_screen = CurrentScreen::Deleting;
                }
                (_, KeyCode::Char('w') | KeyCode::Char('W')) => {
                    self.hide_done().unwrap_or_default()
                }
                (_, KeyCode::Char('h') | KeyCode::Char('H')) => {
                    self.current_screen = CurrentScreen::Help
                }
                (_, KeyCode::Char('l')) => self.loading(),
                _ => {}
            },
            CurrentScreen::Editing => match (key.modifiers, key.code) {
                (_, KeyCode::Enter) => {
                    self.current_screen = CurrentScreen::Main;
                    self.editing = false;
                    match self.current_editing {
                        CurrentEditing::Description => self.tasks[self.state.selected().unwrap()]
                            .set_description(self.buffer.clone()),
                        CurrentEditing::Body => {
                            self.tasks[self.state.selected().unwrap()].set_body(self.buffer.clone())
                        }
                    }
                    self.buffer.clear();
                    self.save_to_file().unwrap();
                }
                (_, KeyCode::Left) => self.move_cursor_left(),
                (_, KeyCode::Right) => self.move_cursor_right(),
                (_, KeyCode::Esc) => {
                    self.current_screen = CurrentScreen::Main;
                    self.editing = false;
                    self.buffer.clear();
                }
                (_, KeyCode::Tab) => {
                    match self.current_editing {
                        CurrentEditing::Description => {
                            self.current_editing = CurrentEditing::Body;
                            self.tasks[self.state.selected().unwrap()]
                                .set_description(self.buffer.clone());
                            self.buffer.clear();
                            self.buffer
                                .push_str(&self.tasks[self.state.selected().unwrap()].body());
                            self.character_index = self.buffer.chars().count();
                        }
                        CurrentEditing::Body => {
                            self.current_editing = CurrentEditing::Description;
                            self.tasks[self.state.selected().unwrap()]
                                .set_body(self.buffer.clone());
                            self.buffer.clear();
                            self.buffer.push_str(
                                &self.tasks[self.state.selected().unwrap()].description(),
                            );
                            self.character_index = self.buffer.chars().count();
                        }
                    }
                    self.save_to_file().unwrap();
                }
                (_, KeyCode::Backspace) => {
                    if !self.buffer.is_empty() {
                        self.delete_char();
                    }
                }
                (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => {
                    self.current_screen = CurrentScreen::Exiting;
                    self.editing = false;
                }
                (_, KeyCode::Char(value)) => {
                    if self.editing {
                        self.enter_char(value);
                    }
                }
                _ => {}
            },
            CurrentScreen::Help => match (key.modifiers, key.code) {
                (_, KeyCode::Esc | KeyCode::Char('q')) => self.current_screen = CurrentScreen::Main,
                (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => {
                    self.current_screen = CurrentScreen::Exiting
                }
                _ => {}
            },
            CurrentScreen::Exiting => match (key.modifiers, key.code) {
                (_, KeyCode::Char('q')) | (_, KeyCode::Char('y')) => self.quit = true,
                (_, KeyCode::Char('n')) | (_, KeyCode::Esc) => {
                    self.current_screen = CurrentScreen::Main
                }
                _ => {}
            },
            CurrentScreen::Deleting => match (key.modifiers, key.code) {
                (_, KeyCode::Char('y') | KeyCode::Char('Y')) => {
                    self.remove_task(self.state.selected().unwrap())
                        .unwrap_or_default();
                    self.current_screen = CurrentScreen::Main;
                }
                (_, KeyCode::Char('n') | KeyCode::Char('N')) => {
                    self.current_screen = CurrentScreen::Main
                }
                _ => {}
            },
        }
    }

    fn loading(&mut self) {
        self.loading = !self.loading;
    }

    pub fn add_task(&mut self, mut task: Task) -> color_eyre::Result<()> {
        if self.with_json {
            task.set_id(self.index());
            self.write_to_json(self.path.clone())?;
        } else {
            let mut file = File::options()
                .write(true)
                .append(true)
                .open(&self.path_bin)?;
            task.set_id(self.index());
            writeln!(file, "{}", task.to_line())?;
            file.flush()?; // ensures writing
        }
        self.tasks.push(task);
        self.state.select(Some(0)); // reset state
        Ok(())
    }

    pub fn index(&self) -> usize {
        self.tasks.len()
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    pub fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    pub fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.buffer.insert(index, new_char);
        self.move_cursor_right();
    }

    pub fn byte_index(&self) -> usize {
        self.buffer
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.buffer.len())
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.buffer.chars().count())
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.character_index != 0;
        if is_not_cursor_leftmost {
            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;
            let before_char_to_delete = self.buffer.chars().take(from_left_to_current_index);
            let after_char_to_delete = self.buffer.chars().skip(current_index);
            self.buffer = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    pub fn read_from_json(path: PathBuf) -> std::io::Result<Vec<Task>> {
        let content = fs::read_to_string(path)?;
        let tasks: Vec<Task> = serde_json::from_str(&content)?;
        Ok(tasks)
    }

    pub fn write_to_json(&self, path: PathBuf) -> std::io::Result<()> {
        let content = serde_json::to_string_pretty(&self.tasks)?;
        fs::write(path, content)
    }

    // TODO: Error handling
    pub fn tasks_into_string(&self) -> String {
        let mut task_str = String::new();
        self.tasks.iter().for_each(|f| {
            task_str.push_str(&format!(
                "{} {} {} \n",
                f.description(),
                is_completed(f.completed()),
                f.body()
            ))
        });
        task_str
    }

    // TODO: Error Handling
    pub fn remove_task(&mut self, index: usize) -> color_eyre::Result<()> {
        self.tasks.remove(index);
        self.save_to_file()?;
        Ok(())
    }

    pub fn clean_tasks(&mut self) -> color_eyre::Result<()> {
        self.tasks.clear();
        if self.with_json {
            self.write_to_json(self.path.clone())?;
        } else {
            if let Some(parent) = self.path_bin.parent() {
                fs::create_dir_all(parent)?;
            }
            File::create(&self.path_bin)?;
        }

        Ok(())
    }

    pub fn change_task_description(
        &mut self,
        index: usize,
        text: String,
    ) -> color_eyre::Result<()> {
        self.tasks[index].set_description(text);
        self.save_to_file()?;
        Ok(())
    }

    pub fn change_task_done(&mut self, index: usize) -> color_eyre::Result<()> {
        self.tasks[index].set_completed();
        self.save_to_file()?;
        Ok(())
    }

    pub fn hide_done(&mut self) -> color_eyre::Result<()> {
        self.show_done = !self.show_done;
        Ok(())
    }

    pub fn remove_trailing_newline(&self) -> color_eyre::Result<()> {
        let file = File::options()
            .read(true)
            .write(true)
            .open(&self.path_bin)?;
        let buf = BufReader::new(file);

        let mut lines: Vec<String> = buf.lines().filter_map(|line| line.ok()).collect();

        if let Some(last) = lines.last() {
            if last.is_empty() {
                lines.pop();
            }
        }
        let mut file = File::create(&self.path_bin)?;
        for line in lines {
            writeln!(file, "{}", line)?;
        }
        Ok(())
    }

    pub fn save_to_file(&self) -> color_eyre::Result<()> {
        if self.with_json {
            self.write_to_json(self.path.clone())?;
        } else {
            let mut file = File::options()
                .read(true)
                .write(true)
                .open(&self.path_bin)?;
            for line in &self.tasks {
                writeln!(
                    file,
                    "{},{},{}",
                    line.get_id(),
                    line.description(),
                    line.completed()
                )?;
            }
            self.remove_trailing_newline()?;
        }
        Ok(())
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.tasks.len() - 1 {
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
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.tasks.len() - 1
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
}
