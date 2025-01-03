use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use model::{common::Task, util::is_completed};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, List, ListState, StatefulWidget, Widget},
    DefaultTerminal, Frame,
};
use std::{
    error::Error,
    fs::File,
    io::{BufRead, BufReader, Read, Write},
};
#[derive(Debug, Default)]
pub struct App {
    tasks: Vec<Task>,
    running: bool,
    show_done: bool, // TODO: better data structure
    state: ListState,
    throbber_state: throbber_widgets_tui::ThrobberState,
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
            true => Ok(App {
                tasks: Vec::new(),
                running: true,
                show_done: false,
                state: ListState::default(),
                throbber_state: throbber_widgets_tui::ThrobberState::default(),
            }),
            false => {
                let mut task_vec = Vec::new();
                for line in buf.lines() {
                    task_vec.push(Task::from_line(line)?);
                }
                Ok(App {
                    tasks: task_vec,
                    running: true,
                    show_done: false,
                    state: ListState::default(),
                    throbber_state: throbber_widgets_tui::ThrobberState::default(),
                })
            }
        }
    }

    pub fn run(&mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        let tick_rate = std::time::Duration::from_millis(250);
        let mut last_tick = std::time::Instant::now();
        while self.running {
            terminal.draw(|frame| self.draw(frame))?;
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

    fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
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
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            (_, KeyCode::Up) => self.previous(),
            (_, KeyCode::Down) => self.next(),
            (_, KeyCode::Left) => self.unselect(),
            (_, KeyCode::Enter) => self
                .change_task_done(self.state.selected().unwrap())
                .unwrap_or_default(),
            (KeyModifiers::CONTROL, KeyCode::Char('h') | KeyCode::Char('H')) => {
                self.hide_done().unwrap_or_default()
            }
            _ => {}
        }
    }

    fn quit(&mut self) {
        self.running = false;
    }

    pub fn add_task(&mut self, mut task: Task) -> color_eyre::Result<()> {
        let mut file = File::options().write(true).append(true).open("user_data")?;
        task.set_id(self.index());
        writeln!(file, "{}", task.to_line())?;
        file.flush()?; // ensures writing
        self.tasks.push(task);
        self.state = ListState::default(); // reset state
        Ok(())
    }

    pub fn index(&self) -> usize {
        self.tasks.len()
    }

    // TODO: Error handling
    pub fn tasks_into_string(&self) -> String {
        let mut task_str = String::new();
        self.tasks.iter().for_each(|f| {
            task_str.push_str(&format!(
                "{} {} \n",
                f.description(),
                is_completed(f.completed())
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
        File::create("user_data")?;
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

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let full = throbber_widgets_tui::Throbber::default()
            .style(ratatui::style::Style::default().fg(ratatui::style::Color::Yellow))
            .throbber_style(
                ratatui::style::Style::default()
                    .fg(ratatui::style::Color::Red)
                    .add_modifier(ratatui::style::Modifier::BOLD),
            )
            .throbber_set(throbber_widgets_tui::BRAILLE_SIX)
            .use_type(throbber_widgets_tui::WhichUse::Spin);
        let items: Vec<String> = if self.show_done {
            self.tasks
                .iter()
                .map(|task| {
                    task.description().clone() + " " + is_completed(task.completed()).as_str()
                })
                .collect::<Vec<String>>()
        } else {
            self.tasks
                .iter()
                .filter(|x| x.completed() == false)
                .map(|task| {
                    task.description().clone() + " " + is_completed(task.completed()).as_str()
                })
                .collect::<Vec<String>>()
        };

        let list = List::new(items)
            .block(
                Block::bordered()
                    .title(Line::from(" Tasks ".bold()).centered())
                    .border_set(border::ROUNDED),
            )
            .highlight_style(Style::new().reversed())
            .highlight_symbol(">> ")
            .repeat_highlight_symbol(true);

        StatefulWidget::render(list, area, buf, &mut self.state);
        StatefulWidget::render(full, area, buf, &mut self.throbber_state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_correct_task() {
        let mut app = App::new().expect("error instanciating app");
        app.clean_tasks().expect("error while removing all tasks");
        assert_eq!(
            app.add_task(
                Task::new(app.tasks.len(), String::from("Hello World"))
                    .expect("error creating new task")
            )
            .unwrap(),
            ()
        );
    }

    #[test]
    fn set_is_done() {
        let mut task = Task::new(0, String::from("Hello World")).expect("error creating new task");
        task.set_completed();
        assert_eq!(task.completed(), true);
    }

    #[test]
    fn change_valid_text() {
        let mut task = Task::new(0, String::from("Hello World")).expect("error creating new task");
        task.change_text(String::from("New Text"))
            .expect("text is empty");
        assert_eq!(task.description(), String::from("New Text"));
    }

    #[test]
    fn display_tasks() {
        let mut app = App::new().expect("error instanciating app");
        app.clean_tasks().expect("error while removing all tasks");
        app.add_task(
            Task::new(app.tasks.len(), String::from("Hello World"))
                .expect("error creating new task"),
        )
        .expect("error while adding a new task");
        assert_eq!(app.tasks_into_string(), "Hello World [] \n")
    }

    #[test]
    fn remove_task() {
        let mut app = App::new().expect("error instanciating app");
        app.clean_tasks().expect("error while removing all tasks");
        app.add_task(
            Task::new(app.tasks.len(), String::from("Hello World"))
                .expect("error creating new task"),
        )
        .expect("error while adding a new task");
        app.remove_task(0).expect("error while removing task");
        assert_eq!(app.tasks_into_string(), "")
    }

    #[test]
    fn remove_all_tasks() {
        let mut app = App::new().expect("error instanciating app");
        app.clean_tasks().expect("error while removing all tasks");
        app.add_task(
            Task::new(app.tasks.len(), String::from("Hello World"))
                .expect("error creating new task"),
        )
        .expect("error while adding a new task");
        app.clean_tasks().expect("error while removing all tasks");
        assert_eq!(app.tasks_into_string(), "")
    }
}
