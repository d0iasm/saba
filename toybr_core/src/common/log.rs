use alloc::string::String;
use alloc::string::ToString;

#[derive(Clone)]
pub enum LogLevel {
    Debug,
    Warning,
    Error,
}

#[derive(Clone)]
pub struct Log {
    level: LogLevel,
    log: String,
}

impl Log {
    pub fn new(level: LogLevel, log: String) -> Self {
        Self { level, log }
    }
}

impl ToString for Log {
    fn to_string(&self) -> String {
        match self.level {
            LogLevel::Debug => format!("[Debug] {}", self.log),
            LogLevel::Warning => format!("[Warning] {}", self.log),
            LogLevel::Error => format!("[Error] {}", self.log),
        }
    }
}
