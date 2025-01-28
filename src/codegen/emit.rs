use super::context::Context;
use crate::parser::VarDeclaration;
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;

pub fn emit_abstract(
    outpath: &PathBuf,
    func_contexts: &Vec<Context>,
    globals: &Vec<VarDeclaration>,
) -> io::Result<()> {
    let mut file = File::create(&outpath)?;
    Ok(())
}

pub fn emit_x86(
    outpath: &PathBuf,
    func_contexts: &Vec<Context>,
    globals: &Vec<VarDeclaration>,
) -> io::Result<()> {
    let mut file = File::create(&outpath)?;
    Ok(())
}

pub fn emit_m6502(
    outpath: &PathBuf,
    func_contexts: &Vec<Context>,
    globals: &Vec<VarDeclaration>,
) -> io::Result<()> {
    let mut file = File::create(&outpath)?;
    Ok(())
}
