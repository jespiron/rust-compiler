use super::context::{AbstractAssemblyInstruction, AsmLabel, Condition, Context, Dest, Operand};
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

fn serialize_condition(condition: &Condition) -> String {
    match condition {
        Condition::Greater => format!("is_g"),
        Condition::Less => format!("is_l"),
        Condition::Equal => format!("is_eq"),
        Condition::NotEqual => format!("is_neq"),
        Condition::GreaterOrEqual => format!("is_geq"),
        Condition::LessOrEqual => format!("is_leq"),
    }
}

fn serialize_label(label: &AsmLabel) -> String {
    format!("L{}", label.0)
}

pub fn emit_abstract(
    outpath: &PathBuf,
    func_contexts: &Vec<Context>,
    _globals: &Vec<VarDeclaration>,
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
                            Token::EqualEqual => "==",
                            Token::Greater => ">",
                            Token::GreaterEqual => ">=",
                            Token::Less => "<",
                            Token::LessEqual => "<=",
                            _ => unimplemented!("Unsupported binary operation {:?}", op),
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
                AbstractAssemblyInstruction::JmpCondition {
                    condition,
                    tgt_true,
                    tgt_false,
                } => {
                    format!(
                        "jmp {} {} {}\n",
                        serialize_condition(condition),
                        serialize_label(tgt_true),
                        serialize_label(tgt_false)
                    )
                }
                AbstractAssemblyInstruction::Compare {
                    left,
                    right,
                    condition,
                } => {
                    format!(
                        "cmp {} {} {}\n",
                        serialize_operand(left),
                        serialize_condition(condition),
                        serialize_operand(right)
                    )
                }
                AbstractAssemblyInstruction::SetIf { dest, condition } => {
                    format!(
                        "set {} {}\n",
                        serialize_dest(dest),
                        serialize_condition(condition)
                    )
                }
                AbstractAssemblyInstruction::Jmp(label) => {
                    format!("jmp {}\n", serialize_label(label))
                }
                AbstractAssemblyInstruction::Lbl(label) => {
                    format!("{}:\n", serialize_label(label))
                }
                AbstractAssemblyInstruction::Return(operand) => {
                    format!("%eax <- {}\nret\n", serialize_operand(operand))
                }
                AbstractAssemblyInstruction::ReturnVoid => {
                    format!("ret\n")
                }
                AbstractAssemblyInstruction::Phi { dest, srcs } => {
                    format!(
                        "phi {} {}\n",
                        serialize_dest(dest),
                        srcs.iter()
                            .map(|(operand, label)| format!(
                                "({}, {})",
                                serialize_operand(operand),
                                serialize_label(label)
                            ))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                }
            };

            file.write_all(line.as_bytes())?;
        }
    }

    Ok(())
}

pub fn emit_x86(
    outpath: &PathBuf,
    _func_contexts: &Vec<Context>,
    _globals: &Vec<VarDeclaration>,
) -> io::Result<()> {
    let _file = File::create(&outpath)?;
    // ...
    // for each context of each function, iterate context.instructions and emit as x86 code
    // ...
    Ok(())
}

pub fn emit_m6502(
    outpath: &PathBuf,
    _func_contexts: &Vec<Context>,
    _globals: &Vec<VarDeclaration>,
) -> io::Result<()> {
    let _file = File::create(&outpath)?;
    // ...
    // for each context of each function, iterate context.instructions and emit as x86 code
    // ...
    Ok(())
}
