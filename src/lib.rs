use std::{fs, error::Error};

mod lexer;
mod parser;

pub struct Config {
    pub path: String,
}

impl Config {
    pub fn new(mut args: std::env::Args) -> Result<Config, &'static str> {
        args.next();

        let path = match args.next() {
            Some(arg) => arg,
            None => return Err("Please provide a file path"),
        };

        Ok(Config { path })
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(config.path)?;

    println!("Contents: {}", contents);

    Ok(())
}
