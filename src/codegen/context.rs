use crate::lexer::Token;
use crate::parser::{Block, Expr, FnDeclaration, Program, Statement, VarDeclaration};
use std::collections::HashMap;

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
    pub name: String,
    /// Abstract assembly instructions this function compiles into
    pub instructions: Vec<AbstractAssemblyInstruction>,
    /// Largest temp number that has not been used
    temp_counter: usize,
    /// Largest label number that has not been used
    label_counter: usize,
    /// Given a variable name, get the associated temp
    var_to_temp: HashMap<String, usize>,
}

impl Context {
    pub fn new(name: &str) -> Self {
        Context {
            name: name.to_string(),
            instructions: Vec::new(),
            temp_counter: 0,
            label_counter: 0,
            var_to_temp: HashMap::new(),
        }
    }

    pub fn generate(&mut self, fn_declaration: &FnDeclaration) {
        for statement in &fn_declaration.body.statements {
            match statement {
                Statement::VarDecl(declr) => {
                    if let Token::Identifier(varname) = &declr.identifier {
                        // Create temp for new variable
                        let dest_temp = self.new_temp();
                        self.var_to_temp.insert(varname.clone(), dest_temp);
                        let dest = Dest::Temp(dest_temp);

                        // Compute the expression, populate in temp
                        let src = self.generate_expr(&declr.value);
                        self.instructions
                            .push(AbstractAssemblyInstruction::Mov { dest, src });
                    } else {
                        panic!("Invalid identifier"); // Better error handling here
                    }
                }
                _ => unimplemented!("Unsupported statement"),
            }
        }
    }

    /// Returns the location that the result is stored in
    fn generate_expr(&mut self, expr: &Expr) -> Operand {
        match expr {
            Expr::Literal(literal) => match literal {
                // TODO: handle Doubles
                Token::Number(num) => Operand::Const(*num as i128),
                _ => panic!("Invalid literal"),
            },
            Expr::Unary(op, src) => {
                let src_operand = self.generate_expr(src);
                let dest_temp = self.new_temp();
                let dest = Dest::Temp(dest_temp);
                self.instructions.push(AbstractAssemblyInstruction::UnOp {
                    op: op.clone(),
                    dest,
                    src: src_operand,
                });
                Operand::Var(Dest::Temp(dest_temp))
            }
            Expr::Binary(left, op, right) => {
                let left_operand = self.generate_expr(left);
                let right_operand = self.generate_expr(right);
                let dest_temp = self.new_temp();
                let dest = Dest::Temp(dest_temp);
                self.instructions.push(AbstractAssemblyInstruction::BinOp {
                    op: op.clone(),
                    dest,
                    src1: left_operand,
                    src2: right_operand,
                });
                Operand::Var(Dest::Temp(dest_temp))
            }
            Expr::Parentheses(expr) => self.generate_expr(expr),
            Expr::Variable(token) => {
                if let Token::Identifier(varname) = token {
                    if let Some(&temp) = self.var_to_temp.get(varname) {
                        Operand::Var(Dest::Temp(temp))
                    } else {
                        panic!("Undefined variable: {}", varname);
                    }
                } else {
                    panic!("Invalid variable token");
                }
            }
            _ => panic!("Unsupported expression"),
        }
    }

    /// Generates a new temp
    fn new_temp(&mut self) -> usize {
        let temp = self.temp_counter.clone();
        self.temp_counter += 1;
        temp
    }

    /// Generates a new label name
    fn new_label(&mut self) -> usize {
        let label = self.label_counter.clone();
        self.label_counter += 1;
        label
    }
}
