use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use model::{common::Task, util::is_completed};
use ratatui::{prelude::Backend, widgets::ListState, Terminal};
use std::{
    error::Error,
    fs::{self, File},
    io::{BufRead, BufReader, Read, Write},
    path::Path,
};

use crate::ui::render;

#[derive(Debug, Default, PartialEq)]
pub enum CurrentScreen {
    #[default]
    Main,
    Editing,
    Help,
    Exiting,
}

#[derive(Debug, Default)]
pub struct App {
    pub tasks: Vec<Task>,
    pub current_screen: CurrentScreen,
    pub quit: bool,
    pub show_done: bool, // TODO: better data structure
    pub loading: bool,
    pub state: ListState,
    pub throbber_state: throbber_widgets_tui::ThrobberState,
    pub path: String,
    pub with_json: bool,
    pub body_input: String,
    pub editing: bool,
    pub character_index: usize,
}

impl App {
    pub fn new() -> Result<App, Box<dyn Error>> {
        let mut user_data = File::options()
            .read(true)
            .append(true)
            .create(true)
            .open("user_data")?;
        let mut buf = String::new();
        user_data.read_to_string(&mut buf)?;
        match buf.is_empty() {
            true => Ok(App::default()),
            false => {
                let mut task_vec = Vec::new();
                for line in buf.lines() {
                    task_vec.push(Task::from_line(line)?);
                }
                Ok(App {
                    tasks: task_vec,
                    state: ListState::default().with_selected(Some(0)),
                    ..Default::default()
                })
            }
        }
    }

    pub fn with_json(path: &str) -> Result<App, Box<dyn Error>> {
        let file = Path::new(path);
        match !file.exists() {
            true => {
                fs::File::create(path)?;
                Ok(App {
                    tasks: Vec::new(),
                    path: path.to_owned(),
                    with_json: true,
                    ..Default::default()
                })
            }
            false => {
                let buf = Self::read_from_json(path)?;
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
                    self.current_screen = CurrentScreen::Editing;
                    self.editing = true;
                    self.body_input
                        .push_str(&self.tasks[self.state.selected().unwrap()].body());
                    self.character_index = self.body_input.chars().count();
                }
                (_, KeyCode::Tab) => self.tasks[self.state.selected().unwrap()].set_completed(),
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
                    self.save_to_file().unwrap();
                    self.tasks[self.state.selected().unwrap()].set_body(self.body_input.clone());
                    self.body_input = String::new();
                }
                (_, KeyCode::Left) => self.move_cursor_left(),
                (_, KeyCode::Right) => self.move_cursor_right(),
                (_, KeyCode::Esc) => {
                    self.current_screen = CurrentScreen::Main;
                    self.editing = false;
                    self.body_input = String::new();
                }
                (_, KeyCode::Tab) => self
                    .change_task_done(self.state.selected().unwrap())
                    .unwrap(),
                (_, KeyCode::Backspace) => {
                    if !self.body_input.is_empty() {
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
        }
    }

    fn loading(&mut self) {
        self.loading = !self.loading;
    }

    pub fn add_task(&mut self, mut task: Task) -> color_eyre::Result<()> {
        if self.with_json {
            task.set_id(self.index());
            self.write_to_json(&self.path)?;
        } else {
            let mut file = File::options().write(true).append(true).open("user_data")?;
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
        self.body_input.insert(index, new_char);
        self.move_cursor_right();
    }

    pub fn byte_index(&self) -> usize {
        self.body_input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.body_input.len())
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.body_input.chars().count())
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.character_index != 0;
        if is_not_cursor_leftmost {
            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;
            let before_char_to_delete = self.body_input.chars().take(from_left_to_current_index);
            let after_char_to_delete = self.body_input.chars().skip(current_index);
            self.body_input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    pub fn read_from_json(path: &str) -> std::io::Result<Vec<Task>> {
        let content = fs::read_to_string(path)?;
        let tasks: Vec<Task> = serde_json::from_str(&content)?;
        Ok(tasks)
    }

    pub fn write_to_json(&self, path: &str) -> std::io::Result<()> {
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
    pub fn remove_task(&mut self, index: u32) -> color_eyre::Result<()> {
        self.tasks.remove(index as usize);
        self.save_to_file()?;
        Ok(())
    }

    pub fn clean_tasks(&mut self) -> color_eyre::Result<()> {
        self.tasks = Vec::new();
        if self.with_json {
            self.write_to_json(&self.path)?;
        } else {
            File::create("user_data")?;
        }

        Ok(())
    }

    pub fn change_task_text(&mut self, index: usize, text: String) -> color_eyre::Result<()> {
        self.tasks[index].change_text(text)?;
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
        let file = File::options().read(true).write(true).open("user_data")?;
        let buf = BufReader::new(file);

        let mut lines: Vec<String> = buf.lines().filter_map(|line| line.ok()).collect();

        if let Some(last) = lines.last() {
            if last.is_empty() {
                lines.pop();
            }
        }
        let mut file = File::create("user_data")?;
        for line in lines {
            writeln!(file, "{}", line)?;
        }
        Ok(())
    }

    pub fn save_to_file(&self) -> color_eyre::Result<()> {
        if self.with_json {
            self.write_to_json(&self.path)?;
        } else {
            let mut file = File::options().read(true).write(true).open("user_data")?;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn tests_cleanup(path: String) {
        if Path::new(&path).exists() {
            fs::remove_file(path).expect("Failed to remove test file");
        }
    }

    #[test]
    fn add_correct_task() {
        tests_cleanup("user_data".to_owned());
        let mut app = App::new().expect("error instanciating app");
        app.clean_tasks().expect("error while removing all tasks");
        assert_eq!(
            app.add_task(
                Task::new(
                    app.tasks.len(),
                    String::from("Hello World"),
                    String::from("hello world")
                )
                .expect("error creating new task")
            )
            .unwrap(),
            ()
        );
    }

    #[test]
    fn set_is_done() {
        let mut task = Task::new(0, String::from("Hello World"), String::from("hello world"))
            .expect("error creating new task");
        task.set_completed();
        assert_eq!(task.completed(), true);
    }

    #[test]
    fn change_valid_text() {
        let mut task = Task::new(0, String::from("Hello World"), String::from("hello world"))
            .expect("error creating new task");
        task.change_text(String::from("New Text"))
            .expect("text is empty");
        assert_eq!(task.description(), String::from("New Text"));
    }

    #[test]
    fn display_tasks() {
        tests_cleanup("user_data".to_owned());
        let mut app = App::new().expect("error instanciating app");
        app.clean_tasks().expect("error while removing all tasks");
        app.add_task(
            Task::new(
                app.tasks.len(),
                String::from("Hello World"),
                String::from("Hello World"),
            )
            .expect("error creating new task"),
        )
        .expect("error while adding a new task");
        assert_eq!(app.tasks_into_string(), "Hello World [] Hello World \n");
    }

    #[test]
    fn remove_task() {
        tests_cleanup("user_data".to_owned());
        let mut app = App::new().expect("error instanciating app");
        app.clean_tasks().expect("error while removing all tasks");
        app.add_task(
            Task::new(
                app.tasks.len(),
                String::from("Hello World"),
                String::from("Hello World"),
            )
            .expect("error creating new task"),
        )
        .expect("error while adding a new task");
        app.remove_task(0).expect("error while removing task");
        assert_eq!(app.tasks_into_string(), "");
    }

    #[test]
    fn remove_all_tasks() {
        tests_cleanup("user_data".to_owned());
        let mut app = App::new().expect("error instanciating app");
        app.clean_tasks().expect("error while removing all tasks");
        app.add_task(
            Task::new(
                app.tasks.len(),
                String::from("Hello World"),
                String::from("Hello World"),
            )
            .expect("error creating new task"),
        )
        .expect("error while adding a new task");
        app.clean_tasks().expect("error while removing all tasks");
        assert_eq!(app.tasks_into_string(), "");
    }
}
