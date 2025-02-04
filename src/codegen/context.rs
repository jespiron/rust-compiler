use crate::lexer::Token;
use crate::parser::{Block, Expr, FnDeclaration, Program, Statement, VarDeclaration};
use std::collections::HashMap;
use std::mem::uninitialized;

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
    Compare {
        left: Operand,
        right: Operand,
        condition: Condition,
    },
    SetIf {
        dest: Dest,
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

#[derive(Debug, Clone)]
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
                let then_label = AsmLabel(self.new_label());
                let end_label = AsmLabel(self.new_label());
                let else_label = if else_branch.is_some() {
                    AsmLabel(self.new_label())
                } else {
                    end_label
                };

                // Generate condition evaluation
                self.generate_condition(condition_expr, then_label, else_label);

                // 2. Generate code for "then" branch
                // If the "else" branch exists, we must jump to end_label when done
                // Otherwise, we can just fall into the end_label
                self.instructions
                    .push(AbstractAssemblyInstruction::Lbl(then_label));
                self.generate_statement(then_branch);
                if has_else {
                    self.instructions
                        .push(AbstractAssemblyInstruction::Jmp(end_label));
                }

                // 3. Next, generate else branch
                if let Some(else_branch) = else_branch {
                    self.instructions
                        .push(AbstractAssemblyInstruction::Lbl(else_label));
                    self.generate_statement(else_branch);
                    self.instructions
                        .push(AbstractAssemblyInstruction::Lbl(end_label));
                }
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

    fn generate_condition(
        &mut self,
        condition_expr: &Expr,
        then_label: AsmLabel,
        else_label: AsmLabel,
    ) {
        match condition_expr {
            Expr::Binary(left, op, right) => {
                let condition = match op {
                    Token::Less => Condition::Less,
                    Token::Greater => Condition::Greater,
                    Token::EqualEqual => Condition::Equal,
                    Token::BangEqual => Condition::NotEqual,
                    Token::LessEqual => Condition::LessOrEqual,
                    Token::GreaterEqual => Condition::GreaterOrEqual,
                    _ => panic!("Unsupported binary operation in condition"),
                };

                let left_op = self.generate_expr(left);
                let right_op = self.generate_expr(right);

                // Emit compare instruction
                self.instructions
                    .push(AbstractAssemblyInstruction::Compare {
                        left: left_op,
                        right: right_op,
                        condition: condition.clone(),
                    });

                // Use direct conditional jump
                self.instructions
                    .push(AbstractAssemblyInstruction::JmpCondition {
                        condition,
                        tgt_true: then_label,
                        tgt_false: else_label,
                    });
            }
            other_expr => {
                let result = self.generate_expr(other_expr);

                // Assume result is a boolean (0 = false, anything else = true)
                self.instructions
                    .push(AbstractAssemblyInstruction::Compare {
                        left: result,
                        right: Operand::Const(0),
                        condition: Condition::NotEqual,
                    });

                self.instructions
                    .push(AbstractAssemblyInstruction::JmpCondition {
                        condition: Condition::NotEqual,
                        tgt_true: then_label,
                        tgt_false: else_label,
                    });
            }
        };
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
                    Token::Greater
                    | Token::Less
                    | Token::EqualEqual
                    | Token::BangEqual
                    | Token::GreaterEqual
                    | Token::LessEqual => {
                        let condition = match op {
                            Token::Greater => Condition::Greater,
                            Token::Less => Condition::Less,
                            Token::EqualEqual => Condition::Equal,
                            Token::BangEqual => Condition::NotEqual,
                            Token::GreaterEqual => Condition::GreaterOrEqual,
                            Token::LessEqual => Condition::LessOrEqual,
                            _ => unreachable!(),
                        };

                        self.instructions
                            .push(AbstractAssemblyInstruction::Compare {
                                left: left_operand,
                                right: right_operand,
                                condition: condition.clone(),
                            });

                        self.instructions.push(AbstractAssemblyInstruction::SetIf {
                            dest: dest.clone(),
                            condition,
                        });

                        return Operand::Var(dest);
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
            Expr::Call(identifier, args) => self.generate_function_call(identifier, args),
            _ => panic!("Unsupported expression"),
        }
    }

    fn generate_function_call(&mut self, identifier: &Expr, args: &Vec<Expr>) -> Operand {
        unimplemented!("Function calls not implemented");
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
