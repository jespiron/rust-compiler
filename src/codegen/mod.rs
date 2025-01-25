use crate::parser::Program;
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;

mod register_allocator;

mod x86_encoding;
use x86_encoding::{Memory, Op, RegOrMem, Register};

pub fn generate_code(program: Program) -> Vec<u8> {
    let ops = Vec::new();
    ops
}

pub fn to_file(ops: Vec<u8>, outpath: PathBuf) -> io::Result<()> {
    let mut file = File::create(&outpath)?;
    file.write_all(&ops)?;
    Ok(())
}
