mod lexer;
mod parser;

use std::env;
use std::error::Error;
use std::fmt;
use std::fs;
use std::io;

fn main() {
    let config = parse_args();
    if let Some(filename) = config.filename {
        match compile_the_thing(&filename) {
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
    } else {
        eprintln!("Usage: <program> <filename>");
    }
}

pub struct Config {
    pub filename: Option<String>,
}

impl Config {
    fn default() -> Self {
        Config {
            filename: None, // Source file to compile.
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
    FileNotFound { filename: String, source: io::Error },
    ParseError { filename: String, message: String },
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompileError::FileNotFound { filename, source } => {
                write!(f, "Failed to open file '{}': {}", filename, source)
            }
            CompileError::ParseError { filename, message } => {
                write!(f, "Error parsing file '{}': {}", filename, message)
            }
        }
    }
}

impl Error for CompileError {}

fn compile_the_thing(filename: &str) -> Result<(), CompileError> {
    let file = fs::File::open(&filename).map_err(|e| CompileError::FileNotFound {
        filename: filename.to_string(),
        source: e,
    })?;

    lexer::tokenize(file);

    // Placeholder for actual parsing logic
    Err(CompileError::ParseError {
        filename: filename.to_string(),
        message: "Parsing not implemented yet".into(),
    })
}
