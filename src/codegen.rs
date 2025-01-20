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

// Convert IR to final bytecode
pub fn generate_code(program: Program) -> Vec<u8> {
    let mut ops = Vec::new();
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
