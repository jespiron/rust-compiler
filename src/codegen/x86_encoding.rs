#[derive(Debug)]
pub enum Op {
    // Data Movement
    Mov(RegOrMem, RegOrMem), // 0x88-0x8B
    Push(RegOrMem),          // 0x50-0x57, 0xFF
    Pop(RegOrMem),           // 0x58-0x5F
    Lea(Register, Memory),   // 0x8D

    // Arithmetic
    Add(RegOrMem, RegOrMem), // 0x00-0x03
    Sub(RegOrMem, RegOrMem), // 0x28-0x2B
    Mul(RegOrMem),           // 0xF6-0xF7 /4
    Div(RegOrMem),           // 0xF6-0xF7 /6
    Inc(RegOrMem),           // 0x40-0x47, 0xFE-0xFF /0
    Dec(RegOrMem),           // 0x48-0x4F, 0xFE-0xFF /1
    Neg(RegOrMem),           // 0xF6-0xF7 /3

    // Control Flow
    Jmp(i32),  // 0xEB, 0xE9
    Je(i32),   // 0x74, 0x0F84
    Jne(i32),  // 0x75, 0x0F85
    Jl(i32),   // 0x7C, 0x0F8C
    Jle(i32),  // 0x7E, 0x0F8E
    Jg(i32),   // 0x7F, 0x0F8F
    Jge(i32),  // 0x7D, 0x0F8D
    Call(i32), // 0xE8
    Ret,       // 0xC3

    // Comparison
    Cmp(RegOrMem, RegOrMem),  // 0x38-0x3B
    Test(RegOrMem, RegOrMem), // 0x84-0x87

    // Stack Frame
    Enter(u16, u8), // 0xC8
    Leave,          // 0xC9

    // String Operations
    Rep,   // 0xF3
    Movsb, // 0xA4
    Movsw, // 0xA5
    Movsd, // 0xA5

    // System
    Int(u8), // 0xCD
    Syscall, // 0x0F05

    // No Operation
    Nop, // 0x90
}

#[derive(Debug)]
pub enum Register {
    // 8-bit registers
    AL,
    BL,
    CL,
    DL,
    AH,
    BH,
    CH,
    DH,
    // 16-bit registers
    AX,
    BX,
    CX,
    DX,
    SP,
    BP,
    SI,
    DI,
    // 32-bit registers
    EAX,
    EBX,
    ECX,
    EDX,
    ESP,
    EBP,
    ESI,
    EDI,
    // 64-bit registers (if needed)
    RAX,
    RBX,
    RCX,
    RDX,
    RSP,
    RBP,
    RSI,
    RDI,
}

#[derive(Debug)]
pub enum RegOrMem {
    Register(Register),
    Memory(Memory),
    Immediate(i32),
}

#[derive(Debug)]
pub struct Memory {
    base: Option<Register>,
    index: Option<Register>,
    scale: Option<u8>, // 1, 2, 4, or 8
    displacement: i32,
}

fn serialize_op(bytes: &mut Vec<u8>, op: Op) {
    match op {
        Op::Nop => {
            bytes.push(0x90);
        }

        Op::Mov(dest, src) => {
            match (&dest, &src) {
                (RegOrMem::Register(rd), RegOrMem::Register(rs)) => {
                    // Register to register
                    bytes.push(0x89);
                    bytes.push(encode_modrm(rd, rs));
                }
                (RegOrMem::Register(rd), RegOrMem::Immediate(imm)) => {
                    // Immediate to register
                    bytes.push(0xB8 + register_index(rd));
                    bytes.extend_from_slice(&imm.to_le_bytes());
                }
                // Add other mov variants as needed
                _ => unimplemented!("Mov variant not implemented"),
            }
        }

        Op::Push(src) => match src {
            RegOrMem::Register(reg) => {
                bytes.push(0x50 + register_index(&reg));
            }
            RegOrMem::Immediate(imm) => {
                if imm >= -128 && imm <= 127 {
                    bytes.push(0x6A);
                    bytes.push(imm as u8);
                } else {
                    bytes.push(0x68);
                    bytes.extend_from_slice(&imm.to_le_bytes());
                }
            }
            _ => unimplemented!("Push variant not implemented"),
        },

        Op::Pop(dest) => match dest {
            RegOrMem::Register(reg) => {
                bytes.push(0x58 + register_index(&reg));
            }
            _ => unimplemented!("Pop variant not implemented"),
        },

        Op::Add(dest, src) => match (&dest, &src) {
            (RegOrMem::Register(rd), RegOrMem::Register(rs)) => {
                bytes.push(0x01);
                bytes.push(encode_modrm(rd, rs));
            }
            (RegOrMem::Register(rd), RegOrMem::Immediate(imm)) => {
                if *imm >= -128 && *imm <= 127 {
                    bytes.push(0x83);
                    bytes.push(encode_modrm_opcode(rd, 0));
                    bytes.push(*imm as u8);
                } else {
                    bytes.push(0x81);
                    bytes.push(encode_modrm_opcode(rd, 0));
                    bytes.extend_from_slice(&imm.to_le_bytes());
                }
            }
            _ => unimplemented!("Add variant not implemented"),
        },

        Op::Jmp(offset) => {
            if offset >= -128 && offset <= 127 {
                bytes.push(0xEB);
                bytes.push(offset as u8);
            } else {
                bytes.push(0xE9);
                bytes.extend_from_slice(&offset.to_le_bytes());
            }
        }

        Op::Call(offset) => {
            bytes.push(0xE8);
            bytes.extend_from_slice(&offset.to_le_bytes());
        }

        Op::Ret => {
            bytes.push(0xC3);
        }

        // Add other operations as needed...
        _ => unimplemented!("Operation not implemented"),
    }
}

fn register_index(reg: &Register) -> u8 {
    match reg {
        Register::AL | Register::AX | Register::EAX | Register::RAX => 0,
        Register::CL | Register::CX | Register::ECX | Register::RCX => 1,
        Register::DL | Register::DX | Register::EDX | Register::RDX => 2,
        Register::BL | Register::BX | Register::EBX | Register::RBX => 3,
        Register::AH | Register::SP | Register::ESP | Register::RSP => 4,
        Register::CH | Register::BP | Register::EBP | Register::RBP => 5,
        Register::DH | Register::SI | Register::ESI | Register::RSI => 6,
        Register::BH | Register::DI | Register::EDI | Register::RDI => 7,
    }
}

fn encode_modrm(reg1: &Register, reg2: &Register) -> u8 {
    0xC0 | (register_index(reg1) << 3) | register_index(reg2)
}

fn encode_modrm_opcode(reg: &Register, opcode: u8) -> u8 {
    0xC0 | (opcode << 3) | register_index(reg)
}
