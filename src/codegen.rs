use crate::parser::Program;
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;

// Defined according to O0 spec:
// https://github.com/jespiron/c0-vm-standards?tab=readme-ov-file#%E5%86%85%E5%AD%98%E6%93%8D%E4%BD%9C%E6%8C%87%E4%BB%A4
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

// Generates O1 instructions from AST representation
/*
TODO: We can introduce an intermediate representation s0.
    Before: `c0 -> o0`
    After: `c0 -> s0 -> o0`

Instead of converting directly to bytecode Vec<Op>,
we can convert to some intermediate representation.
Based on the s0 file format, this representation should break the program
into sections .start, .functions, .constants, .F0, .F1, etc.
*/
pub fn generate_code(ast: Program) -> Vec<Op> {
    vec![]
}

// Serializes to O1 binary, readable by O1 virtual machine
// https://github.com/jespiron/c0-vm-standards/tree/master?tab=readme-ov-file#%E6%96%87%E6%9C%AC%E6%B1%87%E7%BC%96%E4%B8%8E%E4%BA%8C%E8%BF%9B%E5%88%B6%E6%96%87%E4%BB%B6%E8%BD%AC%E6%8D%A2%E7%A4%BA%E4%BE%8B
pub fn to_binary_file(ops: Vec<Op>, outpath: PathBuf) -> io::Result<()> {
    // Create the file at the specified path (overwrite if it exists)
    let mut file = File::create(&outpath)?;

    // Write the content to the file
    file.write_all("hi".as_bytes())?;
    Ok(())
}
