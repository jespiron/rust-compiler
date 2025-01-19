use crate::lexer::Token;
use crate::parser::{Expr, FnDeclaration, Program, Statement, VarDeclaration};
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;

// Defined according to O0 spec:
// https://github.com/jespiron/c0-vm-standards?tab=readme-ov-file#%E5%86%85%E5%AD%98%E6%93%8D%E4%BD%9C%E6%8C%87%E4%BB%A4
#[derive(Debug)]
pub enum Op {
    Nop,             // 0x00
    Bipush(u8),      // 0x01
    Ipush(i32),      // 0x02
    Pop,             // 0x04
    Pop2,            // 0x05
    PopN(u32),       // 0x06
    Dup,             // 0x07
    Dup2,            // 0x08
    LoadC(u16),      // 0x09
    LoadA(u16, u32), // 0x0a
    New,             // 0x0b
    Snew(u32),       // 0x0c
    ILoad,           // 0x10
    DLoad,           // 0x11
    ALoad,           // 0x12
    IALoad,          // 0x18
    DALoad,          // 0x19
    AALoad,          // 0x1a
    IStore,          // 0x20
    DStore,          // 0x21
    AStore,          // 0x22
    IAStore,         // 0x28
    DAStore,         // 0x29
    AAStore,         // 0x2a
    IAdd,            // 0x30
    DAdd,            // 0x31
    ISub,            // 0x34
    DSub,            // 0x35
    IMul,            // 0x38
    DMul,            // 0x39
    IDiv,            // 0x3c
    DDiv,            // 0x3d
    INeg,            // 0x40
    DNeg,            // 0x41
    ICmp,            // 0x44
    DCmp,            // 0x45
    I2D,             // 0x60
    D2I,             // 0x61
    I2C,             // 0x62
    Jmp(u16),        // 0x70
    Je(u16),         // 0x71
    Jne(u16),        // 0x72
    Jl(u16),         // 0x73
    Jge(u16),        // 0x74
    Jg(u16),         // 0x75
    Jle(u16),        // 0x76
    Call(u16),       // 0x80
    Ret,             // 0x88
    IRet,            // 0x89
    DRet,            // 0x8a
    ARet,            // 0x8b
    IPrint,          // 0xa0
    DPrint,          // 0xa1
    CPrint,          // 0xa2
    SPrint,          // 0xa3
    Printl,          // 0xaf
    IScan,           // 0xb0
    DScan,           // 0xb1
    CScan,           // 0xb2
}

// Intermediate representation matching .s0 format
#[derive(Debug)]
struct IR {
    constants: Vec<Constant>,
    start: Vec<Op>,
    functions: Vec<Function>,
}

#[derive(Debug, Clone)]
enum Constant {
    String(String),
    Int(i32),
    Double(f64),
}

#[derive(Debug)]
struct Function {
    name_index: u16,   // Index into constants for function name
    params_count: u16, // Number of parameters
    level: u16,        // Function nesting level
    instructions: Vec<Op>,
}

impl IR {
    fn new() -> Self {
        IR {
            constants: Vec::new(),
            start: Vec::new(),
            functions: Vec::new(),
        }
    }

    // Add constant and return its index
    fn add_constant(&mut self, constant: Constant) -> u16 {
        let index = self.constants.len();
        self.constants.push(constant);
        index as u16
    }
}

// Generates intermediate representation from AST
fn generate_ir(program: Program) -> IR {
    let mut ir = IR::new();

    // Process global variables in .start section
    for var_decl in program.decl {
        process_global_var(&mut ir, var_decl);
    }

    // Process functions
    for fn_decl in program.fns {
        process_function(&mut ir, fn_decl);
    }

    ir
}

fn process_global_var(ir: &mut IR, var_decl: VarDeclaration) {
    match var_decl.value {
        Expr::Literal(token) => {
            // Add value to constants
            let value = parse_literal(&token);
            let const_index = ir.add_constant(value.clone());

            match value {
                // For small integers, use Bipush for efficiency
                // We might be able to do the same for IPush as well
                Constant::Int(i) if i <= 127 && i >= -128 => {
                    ir.start.push(Op::Bipush(i as u8));
                }
                // For other constants, use LoadC
                _ => {
                    ir.start.push(Op::LoadC(const_index));
                }
            }

            // Store the value in global storage
            match var_decl.type_token {
                Token::Int => ir.start.push(Op::IStore),
                Token::Double => ir.start.push(Op::DStore),
                Token::Char => {
                    ir.start.push(Op::I2C);
                    ir.start.push(Op::IStore);
                }
                _ => panic!("Unsupported global variable type"),
            }
        }
        Expr::Binary(left, op, right) => {
            // Process left and right instructions
            process_expression(&mut ir.start, *left);
            process_expression(&mut ir.start, *right);

            // Generate op instruction
            match op {
                Token::Plus => ir.start.push(Op::IAdd),
                Token::Minus => ir.start.push(Op::ISub),
                Token::Star => ir.start.push(Op::IMul),
                Token::Slash => ir.start.push(Op::IDiv),
                _ => panic!("Unsupported binary operator"),
            }

            // Store the result
            match var_decl.type_token {
                Token::Int => ir.start.push(Op::IStore),
                Token::Double => ir.start.push(Op::DStore),
                Token::Char => {
                    ir.start.push(Op::I2C);
                    ir.start.push(Op::IStore);
                }
                _ => panic!("Unsupported global variable type"),
            }
        }
        Expr::Unary(op, expr) => {
            // Process the expression
            process_expression(&mut ir.start, *expr);

            // Generate op instruction
            match op {
                Token::Minus => ir.start.push(Op::INeg),
                Token::Bang => {
                    // For logical not, compare with 0
                    ir.start.push(Op::Bipush(0));
                    ir.start.push(Op::ICmp);
                    ir.start.push(Op::IStore);
                }
                _ => panic!("Unsupported unary operator"),
            }
        }
        Expr::Parentheses(expr) => {
            // Simply process the inner expression
            process_expression(&mut ir.start, *expr);

            // Store the result
            match var_decl.type_token {
                Token::Int => ir.start.push(Op::IStore),
                Token::Double => ir.start.push(Op::DStore),
                Token::Char => {
                    ir.start.push(Op::I2C);
                    ir.start.push(Op::IStore);
                }
                _ => panic!("Unsupported global variable type"),
            }
        }
        _ => panic!("Unsupported global variable initializer"),
    }
}

fn process_function(ir: &mut IR, fn_decl: FnDeclaration) {
    // Add function name to constants
    if let Token::Identifier(name) = fn_decl.identifier {
        let name_index = ir.add_constant(Constant::String(name));
        let mut instructions = Vec::new();

        // Process function body
        for stmt in fn_decl.body.statements {
            process_statement(&mut instructions, stmt);
        }

        // Add function to IR
        ir.functions.push(Function {
            name_index,
            params_count: fn_decl.params.len() as u16,
            level: 1, // For now, assuming all functions are at level 1
            instructions,
        });
    }
}

fn process_statement(instructions: &mut Vec<Op>, stmt: Statement) {
    match stmt {
        Statement::Return(expr) => {
            if let Some(expr) = expr {
                process_expression(instructions, *expr);
                instructions.push(Op::IRet); // Assuming int return for now
            } else {
                instructions.push(Op::Ret);
            }
        }
        _ => todo!("Handle other statement types"),
    }
}

fn process_expression(instructions: &mut Vec<Op>, expr: Expr) {
    match expr {
        Expr::Literal(token) => {
            let value = parse_literal(&token);
            match value {
                /*
                Although the specs put integers in the constants pool,
                I *think* we can exclude them from the pool,
                as they're small enough to be handled directly by Bipush or IPush.
                */
                Constant::Int(i) if i <= 127 && i >= -128 => {
                    instructions.push(Op::Bipush(i as u8));
                }
                Constant::Int(i) => {
                    instructions.push(Op::Ipush(i));
                }
                // TODO: Implement constants pool
                /* Constants like doubles and strings are too large and thus must
                be stored in the constant pool.

                Every time we need a double or string, we need
                    some way of getting the index of the desired value.
                From here, we use `LoadC` to load the value from the constant pool
                */
                Constant::Double(d) => {
                    panic!("Double literals not yet supported in expressions");
                }
                Constant::String(_) => {
                    panic!("String literals not yet supported in expressions");
                }
            }
        }
        Expr::Binary(left, op, right) => {
            process_expression(instructions, *left);
            process_expression(instructions, *right);
            match op {
                Token::Plus => instructions.push(Op::IAdd),
                Token::Minus => instructions.push(Op::ISub),
                Token::Star => instructions.push(Op::IMul),
                Token::Slash => instructions.push(Op::IDiv),
                _ => panic!("Unsupported binary operator"),
            }
        }
        Expr::Unary(op, expr) => {
            process_expression(instructions, *expr);
            match op {
                Token::Minus => instructions.push(Op::INeg),
                Token::Bang => {
                    instructions.push(Op::Bipush(0));
                    instructions.push(Op::ICmp);
                }
                _ => panic!("Unsupported unary operator"),
            }
        }
        Expr::Parentheses(expr) => {
            process_expression(instructions, *expr);
        }
        Expr::Variable(identifier) => {
            match identifier {
                Token::Identifier(_) => {
                    // TODO: Handle globals and use type-specific loads
                    /*
                    For now, we assume all variables are integers and local.
                    In the full implementation, we need to:
                        - Check if the variable is local or global
                        - Use the appropriate load instruction based
                            on type (int, double, char)
                    */
                    instructions.push(Op::ILoad);
                }
                _ => panic!("Invalid identifier token"),
            }
        }
        Expr::Call(callee, args) => {
            // First, evaluate all arguments and push them onto the stack
            for arg in args {
                process_expression(instructions, arg);
            }

            // Now handle the callee
            match *callee {
                Expr::Variable(Token::Identifier(name)) => {
                    // TODO: Create symbol table
                    /*
                    In the full implementation, we need to:
                        1. Look up the function from a symbol table to get its index
                        2. Push the `Call` instruction with the correct function index
                    Assuming we have the function index, it'll be something like:
                        instructions.push(Op::Call(function_index));
                    */
                    panic!("Need symbol table to resolve function call to {}", name);
                }
                _ => panic!("Callee must be a function identifier"),
            }
        }
    }
}

fn parse_literal(token: &Token) -> Constant {
    match token {
        Token::Number(n) => {
            // Check if the number is an integer
            if n.fract() == 0.0 && *n >= (i32::MIN as f64) && *n <= (i32::MAX as f64) {
                Constant::Int(*n as i32)
            } else {
                Constant::Double(*n)
            }
        }
        Token::StringLiteral(s) => Constant::String(s.clone()),
        _ => panic!("Unsupported literal type"),
    }
}

// Convert IR to final bytecode
pub fn generate_code(program: Program) -> Vec<u8> {
    let ir = generate_ir(program);

    let mut ops = Vec::new();

    // Add magic number (43 30 3a 29)
    ops.extend_from_slice(&[0x43, 0x30, 0x3A, 0x29]);

    // Add version (1)
    ops.extend_from_slice(&[0x00, 0x00, 0x00, 0x01]);

    // Write constants
    let constants_count = ir.constants.len();
    ops.extend_from_slice(&((constants_count as u16).to_be_bytes()));

    for constant in ir.constants {
        match constant {
            Constant::String(s) => {
                ops.push(0x00); // String type
                ops.extend_from_slice(&((s.len() as u16).to_be_bytes()));
                ops.extend_from_slice(s.as_bytes());
            }
            Constant::Int(i) => {
                ops.push(0x01); // Int type
                ops.extend_from_slice(&i.to_be_bytes());
            }
            Constant::Double(d) => {
                ops.push(0x02); // Double type
                ops.extend_from_slice(&d.to_be_bytes());
            }
        }
    }

    // Write start code
    ops.extend_from_slice(&((ir.start.len() as u16).to_be_bytes()));
    for op in ir.start {
        serialize_op(&mut ops, op);
    }

    // Write functions
    ops.extend_from_slice(&((ir.functions.len() as u16).to_be_bytes()));
    for func in ir.functions {
        ops.extend_from_slice(&func.name_index.to_be_bytes());
        ops.extend_from_slice(&func.params_count.to_be_bytes());
        ops.extend_from_slice(&func.level.to_be_bytes());
        ops.extend_from_slice(&((func.instructions.len() as u16).to_be_bytes()));

        for op in func.instructions {
            serialize_op(&mut ops, op);
        }
    }

    ops
}

fn serialize_op(bytes: &mut Vec<u8>, op: Op) {
    match op {
        Op::Nop => bytes.push(0x00),
        Op::Bipush(val) => {
            bytes.push(0x01);
            bytes.push(val);
        }
        Op::Ipush(val) => {
            bytes.push(0x02);
            bytes.extend_from_slice(&val.to_be_bytes());
        }
        Op::Pop => bytes.push(0x04),
        Op::Pop2 => bytes.push(0x05),
        Op::PopN(n) => {
            bytes.push(0x06);
            bytes.extend_from_slice(&n.to_be_bytes());
        }
        Op::Dup => bytes.push(0x07),
        Op::Dup2 => bytes.push(0x08),
        Op::LoadC(index) => {
            bytes.push(0x09);
            bytes.extend_from_slice(&index.to_be_bytes());
        }
        Op::LoadA(index, offset) => {
            bytes.push(0x0a);
            bytes.extend_from_slice(&index.to_be_bytes());
            bytes.extend_from_slice(&offset.to_be_bytes());
        }
        Op::New => bytes.push(0x0b),
        Op::Snew(size) => {
            bytes.push(0x0c);
            bytes.extend_from_slice(&size.to_be_bytes());
        }
        Op::ILoad => bytes.push(0x10),
        Op::DLoad => bytes.push(0x11),
        Op::ALoad => bytes.push(0x12),
        Op::IALoad => bytes.push(0x18),
        Op::DALoad => bytes.push(0x19),
        Op::AALoad => bytes.push(0x1a),
        Op::IStore => bytes.push(0x20),
        Op::DStore => bytes.push(0x21),
        Op::AStore => bytes.push(0x22),
        Op::IAStore => bytes.push(0x28),
        Op::DAStore => bytes.push(0x29),
        Op::AAStore => bytes.push(0x2a),
        Op::IAdd => bytes.push(0x30),
        Op::DAdd => bytes.push(0x31),
        Op::ISub => bytes.push(0x34),
        Op::DSub => bytes.push(0x35),
        Op::IMul => bytes.push(0x38),
        Op::DMul => bytes.push(0x39),
        Op::IDiv => bytes.push(0x3c),
        Op::DDiv => bytes.push(0x3d),
        Op::INeg => bytes.push(0x40),
        Op::DNeg => bytes.push(0x41),
        Op::ICmp => bytes.push(0x44),
        Op::DCmp => bytes.push(0x45),
        Op::I2D => bytes.push(0x60),
        Op::D2I => bytes.push(0x61),
        Op::I2C => bytes.push(0x62),
        Op::Jmp(addr)
        | Op::Je(addr)
        | Op::Jne(addr)
        | Op::Jl(addr)
        | Op::Jge(addr)
        | Op::Jg(addr)
        | Op::Jle(addr) => {
            bytes.push(match op {
                Op::Jmp(_) => 0x70,
                Op::Je(_) => 0x71,
                Op::Jne(_) => 0x72,
                Op::Jl(_) => 0x73,
                Op::Jge(_) => 0x74,
                Op::Jg(_) => 0x75,
                Op::Jle(_) => 0x76,
                _ => unreachable!(),
            });
            bytes.extend_from_slice(&addr.to_be_bytes());
        }
        Op::Call(addr) => {
            bytes.push(0x80);
            bytes.extend_from_slice(&addr.to_be_bytes());
        }
        Op::Ret => bytes.push(0x88),
        Op::IRet => bytes.push(0x89),
        Op::DRet => bytes.push(0x8a),
        Op::ARet => bytes.push(0x8b),
        Op::IPrint => bytes.push(0xa0),
        Op::DPrint => bytes.push(0xa1),
        Op::CPrint => bytes.push(0xa2),
        Op::SPrint => bytes.push(0xa3),
        Op::Printl => bytes.push(0xaf),
        Op::IScan => bytes.push(0xb0),
        Op::DScan => bytes.push(0xb1),
        Op::CScan => bytes.push(0xb2),
    }
}

pub fn to_binary_file(ops: Vec<u8>, outpath: PathBuf) -> io::Result<()> {
    let mut file = File::create(&outpath)?;
    file.write_all(&ops)?;
    Ok(())
}
