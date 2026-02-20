use std::fmt;

use colored::Colorize;

pub enum CommandError {
    NumberParsingError(NumberParsingError),
    UnknownCommand,
    InvalidCharacter(char),
}

pub enum NumberParsingError {
    InvalidCharacter(char),
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use CommandError::*;
        let msg = match self {
            UnknownCommand => "Unknown command".to_string(),
            InvalidCharacter(c) => format!("Invalid character: {}", c),
            _ => "todo!()".to_string(),
        };
        write!(f, "{} {}", "ERROR:".red(), msg)
    }
}
