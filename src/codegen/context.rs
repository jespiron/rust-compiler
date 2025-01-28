use crate::lexer::Token;
use crate::parser::{Block, Expr, FnDeclaration, Program, Statement, VarDeclaration};
use std::collections::HashMap;

#[derive(Debug)]
pub struct AbstractAssemblyProgram {
    pub instructions: Vec<AbstractAssemblyInstruction>,
}

#[derive(Debug)]
pub enum AbstractAssemblyInstruction {
    BinOp {
        op: Token,
        dest: Dest,
        src1: Operand,
        src2: Operand,
    },
    UnOp {
        op: Token,
        dest: Dest,
        src: Operand,
    },
    Mov {
        dest: Dest,
        src: Operand,
    },
}

#[derive(Debug)]
pub enum Dest {
    Register(usize),
    Temp(usize),
}

#[derive(Debug)]
pub enum Operand {
    Const(i128),
    Var(Dest),
}

/// Context for a function
pub struct Context {
    /// Name of function this context is for
    name: String,
    /// Largest temp number that has not been used
    temp_counter: usize,
    /// Largest label number that has not been used
    label_counter: usize,
    /// Given a variable name, get the associated temp
    var_to_temp: HashMap<String, Dest>,
}

impl Context {
    pub fn new(name: &str) -> Self {
        Context {
            name: name.to_string(),
            temp_counter: 0,
            label_counter: 0,
            var_to_temp: HashMap::new(),
        }
    }

    pub fn generate(&mut self, fn_declaration: &FnDeclaration) -> AbstractAssemblyProgram {
        let mut instructions = Vec::new();

        AbstractAssemblyProgram { instructions }
    }

    fn new_temp(&mut self) -> String {
        let temp = format!("%t{}", self.temp_counter);
        self.temp_counter += 1;
        temp
    }

    /// Generates a new label name
    fn new_label(&mut self) -> String {
        let label = format!("L{}", self.label_counter);
        self.label_counter += 1;
        label
    }
}
