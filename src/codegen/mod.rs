use crate::lexer::Token;
use crate::parser::Program;
use emit::{emit_abstract, emit_m6502, emit_x86};
use std::io::{self};
use std::path::PathBuf;

mod context;
use context::Context;

mod emit;

pub enum Target {
    AbstractAssembly,
    X86,
    M6502,
}

pub fn generate_code(program: Program, target: Target, outpath: &PathBuf) -> io::Result<()> {
    // Generate function contexts
    let mut func_contexts: Vec<Context> = Vec::new();
    for function in program.fns {
        if let Token::Identifier(fname) = &function.identifier {
            let mut context = Context::new(&fname);
            context.generate(&function);
            func_contexts.push(context);
        }
    }

    // Finally, emit the program based on target
    match target {
        Target::AbstractAssembly => emit_abstract(&outpath, &func_contexts, &program.decl),
        Target::X86 => emit_x86(&outpath, &func_contexts, &program.decl),
        Target::M6502 => emit_m6502(&outpath, &func_contexts, &program.decl),
    }
}
