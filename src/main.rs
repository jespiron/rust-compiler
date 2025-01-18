mod codegen;
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
    BinaryFileGenerationError {
        outpath: String,
        source: io::Error,
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
            CompileError::BinaryFileGenerationError { outpath, source } => {
                write!(
                    f,
                    "Failed to write binary file to '{}': {}",
                    outpath, source
                )
            }
        }
    }
}

impl Error for CompileError {}

fn compile_the_thing(config: Config) -> Result<(), CompileError> {
    match config.filename {
        None => Err(CompileError::InvalidCommand {}),
        Some(filename) => {
            // Construct the full path: src_dir/filename.c0
            let mut path = PathBuf::from(&config.src_dir);
            path.push(&filename);
            path.set_extension("c0");

            // Open the file at the constructed path
            let file = fs::File::open(&path).map_err(|e| CompileError::FileNotFound {
                filename: path.to_string_lossy().into(),
                source: e,
            })?;

            let tokens = lexer::tokenize(file);
            let program = parser::parse(tokens).map_err(|e| CompileError::ParserError {
                filename: filename.to_string(),
                source: e,
            })?;
            let ops = codegen::generate_code(program);

            // Construct the output path: src_dir/target/filename.o0
            let mut outpath = PathBuf::from(&config.src_dir);
            outpath.push("target");
            fs::create_dir_all(&outpath).map_err(|e| CompileError::FileNotFound {
                filename: outpath.to_string_lossy().into(),
                source: e,
            })?;
            outpath.push(&filename);
            outpath.set_extension("o0");

            // Write the output file
            codegen::to_binary_file(ops, outpath.clone()).map_err(|e| {
                CompileError::BinaryFileGenerationError {
                    outpath: outpath.to_string_lossy().into(),
                    source: e,
                }
            })?;

            Ok(())
        }
    }
}
