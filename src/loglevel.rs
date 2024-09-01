use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq, Eq)]
pub enum LogLevel {
    All,
    Debug,
    Info,
    Warning,
    Severe,
    Off,
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::All => f.write_str("ALL"),
            LogLevel::Debug => f.write_str("DEBUG"),
            LogLevel::Info => f.write_str("INFO"),
            LogLevel::Warning => f.write_str("WARNING"),
            LogLevel::Severe => f.write_str("SEVERE"),
            LogLevel::Off => f.write_str("OFF"),
        }
    }
}