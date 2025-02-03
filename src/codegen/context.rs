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
    Cmp {
        left: Operand,
        right: Operand,
        condition: Condition,
    },
    JmpCondition {
        condition: Condition,
        tgt_true: AsmLabel,
        tgt_false: AsmLabel,
    },
    Jmp(AsmLabel),
    Lbl(AsmLabel),
    Phi {
        dest: Dest,
        srcs: Vec<(Operand, AsmLabel)>,
    },
    Return(Operand),
    ReturnVoid,
}

#[derive(Debug, Clone)]
pub enum Dest {
    Register(usize),
    Temp(usize),
}

#[derive(Debug)]
pub enum Operand {
    Const(i128),
    Var(Dest),
}

#[derive(Debug, Clone, Copy)]
pub struct AsmLabel(pub usize);

#[derive(Debug)]
pub enum Condition {
    Greater,
    Less,
    Equal,
    NotEqual,
    GreaterOrEqual,
    LessOrEqual,
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
            /// TODO: if we're converting to SSA, then we'd want to create a new version of each variable
            /// for each assignment, as well as for each branch. Also some way of placing phi nodes
            var_to_temp: HashMap::new(),
        }
    }

    pub fn generate(&mut self, fn_declaration: &FnDeclaration) {
        // Assign parameters to temps
        for (param_idx, param) in fn_declaration.params.iter().enumerate() {
            if let Token::Identifier(param_name) = &param.identifier {
                let dest_temp = self.new_temp();
                self.var_to_temp.insert(param_name.clone(), dest_temp);
            }
        }

        for statement in &fn_declaration.body.statements {
            self.generate_statement(statement);
        }
    }

    fn generate_statement(&mut self, statement: &Statement) {
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
            Statement::If(condition_expr, then_branch, else_branch) => {
                // First, check whether we're generating with or without else branch
                let has_else = else_branch.is_some();

                // 1. Generate code for checking condition
                // If condition does not hold, jump to appropriate label
                // Otherwise, if condition holds, fall into the "then" branch
                let finish_label = AsmLabel(self.new_label());
                let tgt_false = if has_else {
                    let else_label = AsmLabel(self.new_label());
                    else_label
                } else {
                    finish_label
                };

                let condition_result = match self.generate_expr(condition_expr) {
                    Operand::Const(val) => {
                        let dest = Dest::Temp(self.new_temp());
                        self.instructions.push(AbstractAssemblyInstruction::Mov {
                            dest: dest.clone(),
                            src: Operand::Const(val),
                        });
                        dest
                    }
                    Operand::Var(dest) => dest,
                };

                self.instructions
                    .push(AbstractAssemblyInstruction::Test(condition_result.clone()));
                self.instructions
                    .push(AbstractAssemblyInstruction::JmpCondition {
                        condition: condition_result,
                        tgt_false,
                    });

                // 2. Generate code for "then" branch
                // If the "else" branch exists, we must jump to finish_label when done
                // Otherwise, we can just fall into the finish_label
                self.generate_statement(then_branch);
                if has_else {
                    self.instructions
                        .push(AbstractAssemblyInstruction::Jmp(finish_label));
                }

                // 3. Next, generate else branch
                if let Some(else_branch) = else_branch {
                    self.instructions
                        .push(AbstractAssemblyInstruction::Lbl(tgt_false));
                    self.generate_statement(else_branch);
                }

                // End with finish label
                self.instructions
                    .push(AbstractAssemblyInstruction::Lbl(finish_label));
            }
            Statement::Block(block) => {
                // Handle blocks by generating all their statements
                for stmt in &block.statements {
                    self.generate_statement(stmt);
                }
            }
            Statement::Return(value) => {
                if let Some(expr) = value {
                    let operand = self.generate_expr(expr);
                    self.instructions
                        .push(AbstractAssemblyInstruction::Return(operand));
                } else {
                    self.instructions
                        .push(AbstractAssemblyInstruction::ReturnVoid);
                }
            }
            Statement::Expression(expr) => {
                self.generate_expr(expr);
            }
            _ => unimplemented!("Unsupported statement {:?}", statement),
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
            // Basic arithmetic expressions
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
                match op {
                    // TODO: distinguish mutable from immutable variables
                    Token::Equal => {
                        if let Operand::Var(left_dest) = left_operand {
                            self.instructions.push(AbstractAssemblyInstruction::Mov {
                                dest: left_dest,
                                src: right_operand,
                            })
                        } else {
                            panic!("left side of assignment must be variable");
                        }
                    }
                    _ => {
                        self.instructions.push(AbstractAssemblyInstruction::BinOp {
                            op: op.clone(),
                            dest,
                            src1: left_operand,
                            src2: right_operand,
                        });
                    }
                }

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
