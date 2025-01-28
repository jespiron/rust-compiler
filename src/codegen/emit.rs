use super::context::{AbstractAssemblyInstruction, Context, Dest, Operand};
use crate::lexer::Token;
use crate::parser::VarDeclaration;
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;

fn serialize_dest(dest: &Dest) -> String {
    match dest {
        Dest::Register(reg) => format!("({})", reg),
        Dest::Temp(temp) => format!("%t{}", temp),
    }
}

fn serialize_operand(operand: &Operand) -> String {
    match operand {
        Operand::Const(value) => format!("${}", value),
        Operand::Var(dest) => format!("{}", serialize_dest(dest)),
    }
}

pub fn emit_abstract(
    outpath: &PathBuf,
    func_contexts: &Vec<Context>,
    globals: &Vec<VarDeclaration>,
) -> io::Result<()> {
    let mut file = File::create(&outpath)?;
    for context in func_contexts {
        file.write_all(format!(".{}\n", context.name).as_bytes());
        for instruction in &context.instructions {
            let line = match instruction {
                AbstractAssemblyInstruction::BinOp {
                    op,
                    dest,
                    src1,
                    src2,
                } => {
                    format!(
                        "{} <- {} {} {}\n",
                        serialize_dest(&dest),
                        serialize_operand(&src1),
                        match op {
                            Token::Plus => "+",
                            Token::Minus => "-",
                            Token::Star => "*",
                            Token::Slash => "/",
                            _ => unimplemented!("Unsupported binary operation"),
                        },
                        serialize_operand(&src2)
                    )
                }
                AbstractAssemblyInstruction::UnOp { op, dest, src } => {
                    format!(
                        "{} <- {}{}\n",
                        serialize_dest(&dest),
                        match op {
                            Token::Bang => "!",
                            Token::Minus => "-",
                            Token::Tilde => "~",
                            _ => unimplemented!("Unsupported unary operation"),
                        },
                        serialize_operand(&src)
                    )
                }
                AbstractAssemblyInstruction::Mov { dest, src } => {
                    format!("{} <- {}\n", serialize_dest(&dest), serialize_operand(&src))
                }
            };

            file.write_all(line.as_bytes())?;
        }
    }

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
