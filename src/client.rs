use model::{common::Task, util::is_completed};
use std::{
    error::Error,
    fs::File,
    io::{BufRead, BufReader, Read, Write},
};
pub struct App {
    tasks: Vec<Task>,
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
            true => Ok(App { tasks: Vec::new() }),
            false => {
                //lectura
                let mut task_vec = Vec::new();
                for line in buf.lines() {
                    dbg!("{}", line);
                    task_vec.push(Task::from_line(line)?);
                }
                Ok(App { tasks: task_vec })
            }
        }
    }

    pub fn add_task(&mut self, mut task: Task) -> Result<(), Box<dyn Error>> {
        let mut file = File::options().write(true).append(true).open("user_data")?;
        task.set_id(self.index());
        writeln!(file, "{}", task.to_line())?;
        file.flush()?; // ensures writing
        self.tasks.push(task);
        Ok(())
    }

    pub fn index(&self) -> usize {
        self.tasks.len()
    }

    // TODO: Error handling
    pub fn show_tasks(&self) -> String {
        let mut task_str = String::new();
        self.tasks.iter().for_each(|f| {
            task_str.push_str(&format!(
                "> {} {} \n",
                f.description(),
                is_completed(f.completed())
            ))
        });
        task_str
    }

    // TODO: Error Handling
    pub fn remove_task(&mut self, index: u32) -> Result<(), Box<dyn Error>> {
        self.tasks.remove(index as usize);
        self.save_to_file()?;
        Ok(())
    }

    pub fn clean_tasks(&mut self) -> Result<(), Box<dyn Error>> {
        self.tasks = Vec::new();
        File::create("user_data")?;
        Ok(())
    }

    pub fn change_task_text(&mut self, index: usize, text: String) -> Result<(), Box<dyn Error>> {
        self.tasks[index].change_text(text)?;
        self.save_to_file()?;
        Ok(())
    }

    pub fn change_task_done(&mut self, index: usize) -> Result<(), Box<dyn Error>> {
        self.tasks[index].set_completed();
        self.save_to_file()?;
        Ok(())
    }

    pub fn remove_trailing_newline(&self) -> Result<(), Box<dyn Error>> {
        let file = File::options().read(true).write(true).open("user_data")?;
        let buf = BufReader::new(file);

        let mut lines: Vec<String> = buf.lines().filter_map(|line| line.ok()).collect();

        if let Some(last) = lines.last() {
            if last.is_empty() {
                lines.pop();
            }
        }
        // sobreescribir
        let mut file = File::create("user_data")?;
        for line in lines {
            writeln!(file, "{}", line)?;
        }
        Ok(())
    }

    pub fn save_to_file(&self) -> Result<(), Box<dyn Error>> {
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
        assert_eq!(app.show_tasks(), "> Hello World [] \n")
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
        assert_eq!(app.show_tasks(), "")
    }

    #[test]
    fn remove_all_tasks() {
        let mut app = App::new().expect("error instanciating app");
        app.add_task(
            Task::new(app.tasks.len(), String::from("Hello World"))
                .expect("error creating new task"),
        )
        .expect("error while adding a new task");
        app.clean_tasks().expect("error while removing all tasks");
        assert_eq!(app.show_tasks(), "")
    }
}
