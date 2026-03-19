use std::{
    fmt::Display,
    io::{self, ErrorKind},
    str::Utf8Error,
};

use toml::de::Error as TomlError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    FileNotFound,
    PermissionError,
    UnknownError,

    TomlError { err: TomlError },
    NotifyError { err: notify::Error },
    Utf8Error,

    UnknownKeys,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Error::*;
        let string;
        write!(
            f,
            "{}",
            match self {
                FileNotFound => "the settings file was not found in the current directory",
                PermissionError =>
                    "the process didnt had the permission to access the settings file",
                UnknownError => "an unknown error occured in the process",

                TomlError { err } => err.message(),
                NotifyError { err } => {
                    string = err.to_string();
                    &string
                }
                Utf8Error => "the config file contained an UTF-8 error",

                UnknownKeys => "the settings file did contain unknown keys",
            }
        )
    }
}

impl From<TomlError> for Error {
    fn from(value: TomlError) -> Self {
        Self::TomlError { err: value }
    }
}

impl From<notify::Error> for Error {
    fn from(value: notify::Error) -> Self {
        Self::NotifyError { err: value }
    }
}

impl From<Utf8Error> for Error {
    fn from(_: Utf8Error) -> Self {
        Self::Utf8Error
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        match value.kind() {
            ErrorKind::NotFound => Error::FileNotFound,
            ErrorKind::PermissionDenied => Error::PermissionError,
            _ => Error::UnknownError,
        }
    }
}
