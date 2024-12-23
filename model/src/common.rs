use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum TaskError {
    EmptyStringError,
}

// This is required so `TaskError` can implement `Error`.
impl fmt::Display for TaskError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match *self {
            TaskError::EmptyStringError => "text is empty",
        };
        f.write_str(description)
    }
}
// And this is required to use the `?` operator.
impl Error for TaskError {}

#[derive(Debug)]
pub struct Task {
    id: usize,
    description: String,
    completed: bool,
}

impl Task {
    pub fn new(id: usize, str: String) -> Result<Task, TaskError> {
        if str.is_empty() {
            return Err(TaskError::EmptyStringError);
        }
        Ok(Task {
            id,
            description: str,
            completed: false,
        })
    }

    pub fn to_line(&self) -> String {
        format!("{},{},{}", self.id, self.description, self.completed)
    }

    pub fn from_line(line: &str) -> Result<Task, TaskError> {
        let parts: Vec<&str> = line.rsplit(",").collect();
        match parts.len() == 3 {
            true => {
                return Ok(Task {
                    id: parts[2].parse::<usize>().unwrap(),
                    description: parts[1].parse::<String>().unwrap(),
                    completed: parts[0].parse::<bool>().unwrap(),
                });
            }
            false => {
                dbg!("{}", parts.len());
                return Err(TaskError::EmptyStringError);
            }
        }
    }

    pub fn completed(&self) -> bool {
        self.completed
    }

    pub fn get_id(&self) -> usize {
        self.id
    }

    pub fn set_id(&mut self, id: usize) {
        self.id = id;
    }

    pub fn description(&self) -> String {
        self.description.clone()
    }

    pub fn set_completed(&mut self) {
        self.completed = !self.completed;
    }

    pub fn change_text(&mut self, text: String) -> Result<(), TaskError> {
        match text.is_empty() {
            true => return Err(TaskError::EmptyStringError),
            false => {
                self.description = text;
                Ok(())
            }
        }
    }
}
