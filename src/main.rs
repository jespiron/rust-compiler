mod lexer;
mod parser;

use std::env;
use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::path::PathBuf;

fn main() {
    let config = parse_args();

    match compile_the_thing(config) {
        Ok(()) => {
            println!("Compilation succeeded");
        }
        Err(e) => {
            // Pretty print the error
            eprintln!("{}", e);

            // Optionally, print the cause chain for detailed debugging
            let mut source = e.source();
            while let Some(cause) = source {
                eprintln!("Caused by: {}", cause);
                source = cause.source();
            }
        }
    }
}

pub struct Config {
    pub filename: Option<String>,
    pub src_dir: String,
}

impl Config {
    fn default() -> Self {
        Config {
            filename: None, // Source file to compile
            src_dir: String::from("samples"),
        }
    }
}

pub fn parse_args() -> Config {
    let args: Vec<String> = env::args().collect();
    let mut config = Config::default();
    for index in 1..args.len() {
        match args[index].as_str() {
            // Special flags go here
            // Default: treat as filename
            filename => {
                config.filename = Some(filename.to_string());
            }
        }
    }
    config
}

#[derive(Debug)]
enum CompileError {
    InvalidCommand {},
    FileNotFound {
        filename: String,
        source: io::Error,
    },
    ParserError {
        filename: String,
        source: parser::ParserError,
    },
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompileError::InvalidCommand {} => {
                write!(f, "Usage: <program> <filename>")
            }
            CompileError::FileNotFound { filename, source } => {
                write!(f, "Failed to open file '{}': {}", filename, source)
            }
            CompileError::ParserError { filename, source } => {
                write!(f, "Error parsing file '{}': {}", filename, source)
            }
        }
    }
}

impl Error for CompileError {}

fn compile_the_thing(config: Config) -> Result<(), CompileError> {
    match config.filename {
        None => Err(CompileError::InvalidCommand {}),
        Some(filename) => {
            // Construct the full path: src_dir/filename
            let mut path = PathBuf::from(config.src_dir);
            path.push(&filename);

            // Open the file at the constructed path
            let file = fs::File::open(&path).map_err(|e| CompileError::FileNotFound {
                filename: path.to_string_lossy().into(),
                source: e,
            })?;

            let tokens = lexer::tokenize(file);
            let _program = parser::parse(tokens).map_err(|e| CompileError::ParserError {
                filename: filename.to_string(),
                source: e,
            })?;

            Ok(())
        }
    }
}
