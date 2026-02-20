use colored::Colorize;
use num::{BigInt, BigRational, FromPrimitive};
use rustyline::DefaultEditor;
use std::{io, str::Chars, thread};

use crate::{
    console::error::{CommandError, NumberParsingError},
    physics::DeltaTime,
};
mod error;

pub struct Console {
    thread: thread::JoinHandle<()>,
}

impl Console {
    pub fn init(delta_time: DeltaTime) -> Result<Self, io::Error> {
        let thread = thread::Builder::new()
            .name("console".to_string())
            .spawn(move || {
                let mut rl = DefaultEditor::new().unwrap();

                loop {
                    let readline = rl.readline(">> ");
                    match readline {
                        Ok(line) => {
                            _ = rl.add_history_entry(line.as_str());
                            if line.starts_with('/') {
                                // parsing a command:
                                let rest = &line[1..];
                                if rest.len() == 0 {
                                    continue;
                                }
                                match parse_command(rest) {
                                    Ok(command) => {
                                        let Command {
                                            kind: command_type,
                                            args,
                                        } = command;
                                        use CommandType::*;
                                        match command_type {
                                            Status => {
                                                let fps = 1.0 / delta_time.get_f32();
                                                let msg = format!("FPS: {}", fps);
                                                let msg = if fps < 10.0 {
                                                    msg.red()
                                                } else if fps < 60.0 {
                                                    msg.truecolor(200, 180, 0)
                                                } else {
                                                    println!("Everything alright!");
                                                    msg.green()
                                                };
                                                println!("{}", msg.bold())
                                            }
                                            Quit => return,
                                        }
                                    }
                                    Err(err) => println!("{}", err),
                                }
                            } else {
                                // chat message:
                                println!("{} {}", "Mika:".bold(), line)
                            }
                        }
                        Err(_) => break,
                    }
                }
            })?;
        Ok(Console { thread })
    }
}
enum CommandType {
    Status,
    Quit,
}
impl CommandType {
    fn from_str(string: &str) -> Option<Self> {
        use CommandType::*;
        match string {
            "status" => Some(Status),
            "quit" => Some(Quit),
            _ => None,
        }
    }
}
enum Arg {
    String(String),
    Number(u128),
    Coordinate {
        x: BigRational,
        y: BigRational,
        z: BigRational,
    },
}
struct Command {
    kind: CommandType,
    args: Vec<Arg>,
}
fn parse_command(raw_command: &str) -> Result<Command, CommandError> {
    let mut chars = raw_command.chars();
    let mut command_name = String::new();
    while let Some(c) = chars.next() {
        if c.is_ascii_alphanumeric() {
            command_name.push(c)
        } else if let ' ' | '\n' | '\t' = c {
            break;
        } else {
            return Err(CommandError::InvalidCharacter(c));
        }
    }
    let Some(command_type) = CommandType::from_str(&command_name) else {
        return Err(CommandError::UnknownCommand);
    };
    while let Some(c) = chars.next() {
        if c.is_ascii_digit() {
            match parse_number(c, &mut chars) {
                Ok(num) => todo!(),
                Err(err) => todo!(),
            }
        }
    }
    return Ok(Command {
        kind: command_type,
        args: vec![],
    });
}
fn parse_number(first_char: char, chars: &mut Chars) -> Result<BigRational, NumberParsingError> {
    let base: Base;
    let mut result = BigRational::new(first_char.to_digit(10).unwrap().into(), 1.into());
    if first_char == '0' {
        let Some(second_char) = chars.next() else {
            return Ok(BigRational::from_u8(0).unwrap()); // its just zero
        };
        match second_char {
            'b' => base = Base::Binary,
            's' => base = Base::Seximal,
            'o' => base = Base::Octal,
            'd' => base = Base::Dozenal,
            'x' => base = Base::Hexadecimal,
            _ => {
                if let Some(num) = second_char.to_digit(10) {
                    base = Base::Decimal;
                    result = BigRational::from_u32(num).unwrap();
                } else {
                    return Err(NumberParsingError::InvalidCharacter(second_char));
                }
            }
        }
    } else {
        base = Base::Decimal
    }
    let mut after_decimal_point = false;
    while let Some(c) = chars.next() {
        if let Some(num) = c.to_digit(base as u32) {
            let (numer, mut denom) = result.into_raw();
            if after_decimal_point {
                denom = denom * BigInt::from_u32(base as u32).unwrap()
            }
            result = BigRational::new(
                numer * BigInt::from(base as usize) + BigInt::from(num),
                denom,
            )
        } else
        // the character doesnt match the base
        if c == '.' {
            after_decimal_point = true;
        } else if c != '_' {
            return Err(NumberParsingError::InvalidCharacter(c));
        }
    }
    Ok(result)
}
#[derive(Clone, Copy, PartialEq)]
enum Base {
    Binary = 2,
    Seximal = 6,
    Octal = 8,
    Decimal = 10,
    Dozenal = 12,
    Hexadecimal = 16,
}
