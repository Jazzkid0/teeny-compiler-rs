use clap::{Parser, Subcommand};
use std::{
    fs,
    string::String,
};

mod lexer;
mod parser;
mod emitter;

#[derive(Parser, Debug)]
#[command(name = "teeny compiler", version, about = "Simple compiler for a BASIC-like grammar into C", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
#[command(version, about, long_about = None)]
enum Command {
    /// Compile a single .tiny file
    #[command()]
    Compile { path: String },
}

fn main() {
    let args = Cli::parse();
    let target_dir = "./tinycode/";

    match args.command {
        Command::Compile { path } => {
            let input = fs::read_to_string(path).unwrap();
            println!("{}", input);
            let lex_out = lexer::lex(&input).unwrap();
            let mut token_iterator = lexer::TokenIterator::new(&lex_out).peekable();
            let parse_out = parser::parse(&mut token_iterator).unwrap();
            let parser::AST::Program(statements) = parse_out;
            let output = emitter::emit_program(statements).unwrap();
            for line in output {
                println!("{}", line);
            }
        }
    }
}
