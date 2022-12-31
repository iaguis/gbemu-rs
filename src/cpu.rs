use std::thread;
use std::time;
use std::fs::File;

use crate::registers::{Flag,Registers};
use crate::memory_bus::MemoryBus;

pub struct CPU {
    pub reg: Registers,
    pub memory_bus: MemoryBus,
    pub counter: i32,
    pub tmp_buffer: Vec<u8>,
    // is this all we need for HALT?
    running: bool,
    IME: bool,
}

#[repr(u8)]
#[derive(Debug)]
pub enum Opcode {
    NOP,
    LD(LDType),
    INC(IncDecTarget),
    DEC(IncDecTarget),
    PUSH(StackTarget),
    POP(StackTarget),
    JP(JCondition),
    JPI,
    JR(JCondition),

    ADD(ALUOperand),
    ADDHL(ADDHLOperand),
    ADDSP,
    ADC(ALUOperand),
    SUB(ALUOperand),
    SBC(ALUOperand),
    AND(ALUOperand),
    OR(ALUOperand),
    XOR(ALUOperand),
    CP(ALUOperand),

    CPL,
    CCF,
    SCF,
    DAA,

    RLCA,
    RLA,
    RRCA,
    RRA,

    CALL(JCondition),
    RET(JCondition),
    RETI,
    RST(RSTAddress),

    DI,
    HALT,
    EI,

    PREFIX,
}

#[repr(u8)]
#[derive(Debug)]
pub enum PrefixedOpcode {
    RLC(RotateOperand),
}

#[derive(Debug)]
pub enum RotateOperand {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    HLIndirect,
}

#[derive(Debug)]
pub enum ALUOperand {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    D8,
    HLIndirect,
}

#[derive(Debug)]
pub enum ADDHLOperand {
    BC,
    DE,
    HL,
    SP,
}

#[derive(Debug)]
pub enum RSTAddress {
    X00,
    X10,
    X20,
    X30,
    X08,
    X18,
    X28,
    X38,
}

#[derive(Debug)]
pub enum JCondition {
    Nothing,
    NZ,
    NC,
    Z,
    C,
}

#[derive(Debug)]
pub enum LDType {
    Byte(LDTarget, LDSource),
    Word(LDWordTarget),
    AFromIndirect(Indirect),
    IndirectFromA(Indirect),
    AFromAddress,
    AddressFromA,
    SPFromHL,
    IndirectFromSP,
}

#[derive(Debug)]
pub enum IncDecTarget {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    BC,
    DE,
    HL,
    SP,
    HLIndirect,
}

#[derive(Debug)]
pub enum Indirect {
    BCIndirect,
    DEIndirect,
    HLIndirectInc,
    HLIndirectDec,
    WordIndirect,
    LastByteIndirect,
}

#[derive(Debug)]
pub enum StackTarget {
    AF,
    BC,
    DE,
    HL,
}

#[derive(Debug)]
pub enum LDWordTarget {
    BC,
    DE,
    HL,
    SP,
}

#[derive(Debug)]
pub enum LDSource {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    D8,
    HLIndirect,
}

#[derive(Debug)]
pub enum LDTarget {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    HLIndirect,
}

impl TryFrom<u8> for Opcode {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Opcode::NOP),
            0x01 => Ok(Opcode::LD(LDType::Word(LDWordTarget::BC))),
            0x02 => Ok(Opcode::LD(LDType::IndirectFromA(Indirect::BCIndirect))),
            0x03 => Ok(Opcode::INC(IncDecTarget::BC)),
            0x04 => Ok(Opcode::INC(IncDecTarget::B)),
            0x05 => Ok(Opcode::DEC(IncDecTarget::B)),
            0x06 => Ok(Opcode::LD(LDType::Byte(LDTarget::B, LDSource::D8))),
            0x07 => Ok(Opcode::RLCA),
            0x08 => Ok(Opcode::LD(LDType::IndirectFromSP)),
            0x09 => Ok(Opcode::ADDHL(ADDHLOperand::BC)),
            0x0a => Ok(Opcode::LD(LDType::AFromIndirect(Indirect::BCIndirect))),
            0x0b => Ok(Opcode::DEC(IncDecTarget::BC)),
            0x0c => Ok(Opcode::INC(IncDecTarget::C)),
            0x0d => Ok(Opcode::DEC(IncDecTarget::C)),
            0x0e => Ok(Opcode::LD(LDType::Byte(LDTarget::C, LDSource::D8))),
            0x0f => Ok(Opcode::RRCA),
            // 0x10 STOP. Not used by commercial games.
            0x11 => Ok(Opcode::LD(LDType::Word(LDWordTarget::DE))),
            0x12 => Ok(Opcode::LD(LDType::IndirectFromA(Indirect::DEIndirect))),
            0x13 => Ok(Opcode::INC(IncDecTarget::DE)),
            0x14 => Ok(Opcode::INC(IncDecTarget::D)),
            0x15 => Ok(Opcode::DEC(IncDecTarget::D)),
            0x16 => Ok(Opcode::LD(LDType::Byte(LDTarget::D, LDSource::D8))),
            0x17 => Ok(Opcode::RLA),
            0x18 => Ok(Opcode::JR(JCondition::Nothing)),
            0x19 => Ok(Opcode::ADDHL(ADDHLOperand::DE)),
            0x1a => Ok(Opcode::LD(LDType::AFromIndirect(Indirect::DEIndirect))),
            0x1b => Ok(Opcode::DEC(IncDecTarget::DE)),
            0x1c => Ok(Opcode::INC(IncDecTarget::E)),
            0x1d => Ok(Opcode::DEC(IncDecTarget::E)),
            0x1e => Ok(Opcode::LD(LDType::Byte(LDTarget::E, LDSource::D8))),
            0x1f => Ok(Opcode::RRA),
            0x20 => Ok(Opcode::JR(JCondition::NZ)),
            0x21 => Ok(Opcode::LD(LDType::Word(LDWordTarget::HL))),
            0x22 => Ok(Opcode::LD(LDType::IndirectFromA(Indirect::HLIndirectInc))),
            0x23 => Ok(Opcode::INC(IncDecTarget::HL)),
            0x24 => Ok(Opcode::INC(IncDecTarget::H)),
            0x25 => Ok(Opcode::DEC(IncDecTarget::H)),
            0x26 => Ok(Opcode::LD(LDType::Byte(LDTarget::H, LDSource::D8))),
            0x27 => Ok(Opcode::DAA),
            0x28 => Ok(Opcode::JR(JCondition::Z)),
            0x29 => Ok(Opcode::ADDHL(ADDHLOperand::HL)),
            0x2a => Ok(Opcode::LD(LDType::AFromIndirect(Indirect::HLIndirectInc))),
            0x2b => Ok(Opcode::DEC(IncDecTarget::HL)),
            0x2c => Ok(Opcode::INC(IncDecTarget::L)),
            0x2d => Ok(Opcode::DEC(IncDecTarget::L)),
            0x2e => Ok(Opcode::LD(LDType::Byte(LDTarget::L, LDSource::D8))),
            0x2f => Ok(Opcode::CPL),
            0x30 => Ok(Opcode::JR(JCondition::NC)),
            0x31 => Ok(Opcode::LD(LDType::Word(LDWordTarget::SP))),
            0x32 => Ok(Opcode::LD(LDType::IndirectFromA(Indirect::HLIndirectDec))),
            0x33 => Ok(Opcode::INC(IncDecTarget::SP)),
            0x34 => Ok(Opcode::INC(IncDecTarget::HLIndirect)),
            0x35 => Ok(Opcode::DEC(IncDecTarget::HLIndirect)),
            0x36 => Ok(Opcode::LD(LDType::Byte(LDTarget::HLIndirect, LDSource::D8))),
            0x37 => Ok(Opcode::SCF),
            0x38 => Ok(Opcode::JR(JCondition::C)),
            0x39 => Ok(Opcode::ADDHL(ADDHLOperand::SP)),
            0x3a => Ok(Opcode::LD(LDType::AFromIndirect(Indirect::HLIndirectDec))),
            0x3b => Ok(Opcode::DEC(IncDecTarget::SP)),
            0x3c => Ok(Opcode::INC(IncDecTarget::A)),
            0x3d => Ok(Opcode::DEC(IncDecTarget::A)),
            0x3e => Ok(Opcode::LD(LDType::Byte(LDTarget::A, LDSource::D8))),
            0x3f => Ok(Opcode::CCF),
            0x40 => Ok(Opcode::LD(LDType::Byte(LDTarget::B, LDSource::B))),
            0x41 => Ok(Opcode::LD(LDType::Byte(LDTarget::B, LDSource::C))),
            0x42 => Ok(Opcode::LD(LDType::Byte(LDTarget::B, LDSource::D))),
            0x43 => Ok(Opcode::LD(LDType::Byte(LDTarget::B, LDSource::E))),
            0x44 => Ok(Opcode::LD(LDType::Byte(LDTarget::B, LDSource::H))),
            0x45 => Ok(Opcode::LD(LDType::Byte(LDTarget::B, LDSource::L))),
            0x46 => Ok(Opcode::LD(LDType::Byte(LDTarget::B, LDSource::HLIndirect))),
            0x47 => Ok(Opcode::LD(LDType::Byte(LDTarget::B, LDSource::A))),
            0x48 => Ok(Opcode::LD(LDType::Byte(LDTarget::C, LDSource::B))),
            0x49 => Ok(Opcode::LD(LDType::Byte(LDTarget::C, LDSource::C))),
            0x4a => Ok(Opcode::LD(LDType::Byte(LDTarget::C, LDSource::D))),
            0x4b => Ok(Opcode::LD(LDType::Byte(LDTarget::C, LDSource::E))),
            0x4c => Ok(Opcode::LD(LDType::Byte(LDTarget::C, LDSource::H))),
            0x4d => Ok(Opcode::LD(LDType::Byte(LDTarget::C, LDSource::L))),
            0x4e => Ok(Opcode::LD(LDType::Byte(LDTarget::C, LDSource::HLIndirect))),
            0x4f => Ok(Opcode::LD(LDType::Byte(LDTarget::C, LDSource::A))),
            0x50 => Ok(Opcode::LD(LDType::Byte(LDTarget::D, LDSource::B))),
            0x51 => Ok(Opcode::LD(LDType::Byte(LDTarget::D, LDSource::C))),
            0x52 => Ok(Opcode::LD(LDType::Byte(LDTarget::D, LDSource::D))),
            0x53 => Ok(Opcode::LD(LDType::Byte(LDTarget::D, LDSource::E))),
            0x54 => Ok(Opcode::LD(LDType::Byte(LDTarget::D, LDSource::H))),
            0x55 => Ok(Opcode::LD(LDType::Byte(LDTarget::D, LDSource::L))),
            0x56 => Ok(Opcode::LD(LDType::Byte(LDTarget::D, LDSource::HLIndirect))),
            0x57 => Ok(Opcode::LD(LDType::Byte(LDTarget::D, LDSource::A))),
            0x58 => Ok(Opcode::LD(LDType::Byte(LDTarget::E, LDSource::B))),
            0x59 => Ok(Opcode::LD(LDType::Byte(LDTarget::E, LDSource::C))),
            0x5a => Ok(Opcode::LD(LDType::Byte(LDTarget::E, LDSource::D))),
            0x5b => Ok(Opcode::LD(LDType::Byte(LDTarget::E, LDSource::E))),
            0x5c => Ok(Opcode::LD(LDType::Byte(LDTarget::E, LDSource::H))),
            0x5d => Ok(Opcode::LD(LDType::Byte(LDTarget::E, LDSource::L))),
            0x5e => Ok(Opcode::LD(LDType::Byte(LDTarget::E, LDSource::HLIndirect))),
            0x5f => Ok(Opcode::LD(LDType::Byte(LDTarget::E, LDSource::A))),
            0x60 => Ok(Opcode::LD(LDType::Byte(LDTarget::H, LDSource::B))),
            0x61 => Ok(Opcode::LD(LDType::Byte(LDTarget::H, LDSource::C))),
            0x62 => Ok(Opcode::LD(LDType::Byte(LDTarget::H, LDSource::D))),
            0x63 => Ok(Opcode::LD(LDType::Byte(LDTarget::H, LDSource::E))),
            0x64 => Ok(Opcode::LD(LDType::Byte(LDTarget::H, LDSource::H))),
            0x65 => Ok(Opcode::LD(LDType::Byte(LDTarget::H, LDSource::L))),
            0x66 => Ok(Opcode::LD(LDType::Byte(LDTarget::H, LDSource::HLIndirect))),
            0x67 => Ok(Opcode::LD(LDType::Byte(LDTarget::H, LDSource::A))),
            0x68 => Ok(Opcode::LD(LDType::Byte(LDTarget::L, LDSource::B))),
            0x69 => Ok(Opcode::LD(LDType::Byte(LDTarget::L, LDSource::C))),
            0x6a => Ok(Opcode::LD(LDType::Byte(LDTarget::L, LDSource::D))),
            0x6b => Ok(Opcode::LD(LDType::Byte(LDTarget::L, LDSource::E))),
            0x6c => Ok(Opcode::LD(LDType::Byte(LDTarget::L, LDSource::H))),
            0x6d => Ok(Opcode::LD(LDType::Byte(LDTarget::L, LDSource::L))),
            0x6e => Ok(Opcode::LD(LDType::Byte(LDTarget::L, LDSource::HLIndirect))),
            0x6f => Ok(Opcode::LD(LDType::Byte(LDTarget::L, LDSource::A))),
            0x70 => Ok(Opcode::LD(LDType::Byte(LDTarget::HLIndirect, LDSource::B))),
            0x71 => Ok(Opcode::LD(LDType::Byte(LDTarget::HLIndirect, LDSource::C))),
            0x72 => Ok(Opcode::LD(LDType::Byte(LDTarget::HLIndirect, LDSource::D))),
            0x73 => Ok(Opcode::LD(LDType::Byte(LDTarget::HLIndirect, LDSource::E))),
            0x74 => Ok(Opcode::LD(LDType::Byte(LDTarget::HLIndirect, LDSource::H))),
            0x75 => Ok(Opcode::LD(LDType::Byte(LDTarget::HLIndirect, LDSource::L))),
            0x76 => Ok(Opcode::HALT),
            0x77 => Ok(Opcode::LD(LDType::Byte(LDTarget::HLIndirect, LDSource::A))),
            0x78 => Ok(Opcode::LD(LDType::Byte(LDTarget::A, LDSource::B))),
            0x79 => Ok(Opcode::LD(LDType::Byte(LDTarget::A, LDSource::C))),
            0x7a => Ok(Opcode::LD(LDType::Byte(LDTarget::A, LDSource::D))),
            0x7b => Ok(Opcode::LD(LDType::Byte(LDTarget::A, LDSource::E))),
            0x7c => Ok(Opcode::LD(LDType::Byte(LDTarget::A, LDSource::H))),
            0x7d => Ok(Opcode::LD(LDType::Byte(LDTarget::A, LDSource::L))),
            0x7e => Ok(Opcode::LD(LDType::Byte(LDTarget::A, LDSource::HLIndirect))),
            0x7f => Ok(Opcode::LD(LDType::Byte(LDTarget::A, LDSource::A))),
            0x80 => Ok(Opcode::ADD(ALUOperand::B)),
            0x81 => Ok(Opcode::ADD(ALUOperand::C)),
            0x82 => Ok(Opcode::ADD(ALUOperand::D)),
            0x83 => Ok(Opcode::ADD(ALUOperand::E)),
            0x84 => Ok(Opcode::ADD(ALUOperand::H)),
            0x85 => Ok(Opcode::ADD(ALUOperand::L)),
            0x86 => Ok(Opcode::ADD(ALUOperand::HLIndirect)),
            0x87 => Ok(Opcode::ADD(ALUOperand::A)),
            0x88 => Ok(Opcode::ADC(ALUOperand::B)),
            0x89 => Ok(Opcode::ADC(ALUOperand::C)),
            0x8a => Ok(Opcode::ADC(ALUOperand::D)),
            0x8b => Ok(Opcode::ADC(ALUOperand::E)),
            0x8c => Ok(Opcode::ADC(ALUOperand::H)),
            0x8d => Ok(Opcode::ADC(ALUOperand::L)),
            0x8e => Ok(Opcode::ADC(ALUOperand::HLIndirect)),
            0x8f => Ok(Opcode::ADC(ALUOperand::A)),
            0x90 => Ok(Opcode::SUB(ALUOperand::B)),
            0x91 => Ok(Opcode::SUB(ALUOperand::C)),
            0x92 => Ok(Opcode::SUB(ALUOperand::D)),
            0x93 => Ok(Opcode::SUB(ALUOperand::E)),
            0x94 => Ok(Opcode::SUB(ALUOperand::H)),
            0x95 => Ok(Opcode::SUB(ALUOperand::L)),
            0x96 => Ok(Opcode::SUB(ALUOperand::HLIndirect)),
            0x97 => Ok(Opcode::SUB(ALUOperand::A)),
            0x98 => Ok(Opcode::SBC(ALUOperand::B)),
            0x99 => Ok(Opcode::SBC(ALUOperand::C)),
            0x9a => Ok(Opcode::SBC(ALUOperand::D)),
            0x9b => Ok(Opcode::SBC(ALUOperand::E)),
            0x9c => Ok(Opcode::SBC(ALUOperand::H)),
            0x9d => Ok(Opcode::SBC(ALUOperand::L)),
            0x9e => Ok(Opcode::SBC(ALUOperand::HLIndirect)),
            0x9f => Ok(Opcode::SBC(ALUOperand::A)),
            0xa0 => Ok(Opcode::AND(ALUOperand::B)),
            0xa1 => Ok(Opcode::AND(ALUOperand::C)),
            0xa2 => Ok(Opcode::AND(ALUOperand::D)),
            0xa3 => Ok(Opcode::AND(ALUOperand::E)),
            0xa4 => Ok(Opcode::AND(ALUOperand::H)),
            0xa5 => Ok(Opcode::AND(ALUOperand::L)),
            0xa6 => Ok(Opcode::AND(ALUOperand::HLIndirect)),
            0xa7 => Ok(Opcode::AND(ALUOperand::A)),
            0xa8 => Ok(Opcode::XOR(ALUOperand::B)),
            0xa9 => Ok(Opcode::XOR(ALUOperand::C)),
            0xaa => Ok(Opcode::XOR(ALUOperand::D)),
            0xab => Ok(Opcode::XOR(ALUOperand::E)),
            0xac => Ok(Opcode::XOR(ALUOperand::H)),
            0xad => Ok(Opcode::XOR(ALUOperand::L)),
            0xae => Ok(Opcode::XOR(ALUOperand::HLIndirect)),
            0xaf => Ok(Opcode::XOR(ALUOperand::A)),
            0xb0 => Ok(Opcode::OR(ALUOperand::B)),
            0xb1 => Ok(Opcode::OR(ALUOperand::C)),
            0xb2 => Ok(Opcode::OR(ALUOperand::D)),
            0xb3 => Ok(Opcode::OR(ALUOperand::E)),
            0xb4 => Ok(Opcode::OR(ALUOperand::H)),
            0xb5 => Ok(Opcode::OR(ALUOperand::L)),
            0xb6 => Ok(Opcode::OR(ALUOperand::HLIndirect)),
            0xb7 => Ok(Opcode::OR(ALUOperand::A)),
            0xb8 => Ok(Opcode::CP(ALUOperand::B)),
            0xb9 => Ok(Opcode::CP(ALUOperand::C)),
            0xba => Ok(Opcode::CP(ALUOperand::D)),
            0xbb => Ok(Opcode::CP(ALUOperand::E)),
            0xbc => Ok(Opcode::CP(ALUOperand::H)),
            0xbd => Ok(Opcode::CP(ALUOperand::L)),
            0xbe => Ok(Opcode::CP(ALUOperand::HLIndirect)),
            0xbf => Ok(Opcode::CP(ALUOperand::A)),
            0xc0 => Ok(Opcode::RET(JCondition::NZ)),
            0xc1 => Ok(Opcode::POP(StackTarget::BC)),
            0xc2 => Ok(Opcode::JP(JCondition::NZ)),
            0xc3 => Ok(Opcode::JP(JCondition::Nothing)),
            0xc4 => Ok(Opcode::CALL(JCondition::NZ)),
            0xc5 => Ok(Opcode::PUSH(StackTarget::BC)),
            0xc6 => Ok(Opcode::ADD(ALUOperand::D8)),
            0xc7 => Ok(Opcode::RST(RSTAddress::X00)),
            0xc8 => Ok(Opcode::RET(JCondition::Z)),
            0xc9 => Ok(Opcode::RET(JCondition::Nothing)),
            0xca => Ok(Opcode::JP(JCondition::Z)),
            0xcb => Ok(Opcode::PREFIX),
            0xcc => Ok(Opcode::CALL(JCondition::Z)),
            0xcd => Ok(Opcode::CALL(JCondition::Nothing)),
            0xce => Ok(Opcode::ADC(ALUOperand::D8)),
            0xcf => Ok(Opcode::RST(RSTAddress::X08)),
            0xd0 => Ok(Opcode::RET(JCondition::NC)),
            0xd1 => Ok(Opcode::POP(StackTarget::DE)),
            0xd2 => Ok(Opcode::JP(JCondition::NC)),
            0xd4 => Ok(Opcode::CALL(JCondition::NC)),
            0xd5 => Ok(Opcode::PUSH(StackTarget::DE)),
            0xd6 => Ok(Opcode::SUB(ALUOperand::D8)),
            0xd7 => Ok(Opcode::RST(RSTAddress::X10)),
            0xd8 => Ok(Opcode::RET(JCondition::C)),
            0xd9 => Ok(Opcode::RETI),
            0xda => Ok(Opcode::JP(JCondition::C)),
            0xdc => Ok(Opcode::CALL(JCondition::C)),
            0xde => Ok(Opcode::SBC(ALUOperand::D8)),
            0xdf => Ok(Opcode::RST(RSTAddress::X18)),
            0xe0 => Ok(Opcode::LD(LDType::AFromAddress)),
            0xe1 => Ok(Opcode::POP(StackTarget::HL)),
            0xe2 => Ok(Opcode::LD(LDType::IndirectFromA(Indirect::LastByteIndirect))),
            0xe5 => Ok(Opcode::PUSH(StackTarget::HL)),
            0xe6 => Ok(Opcode::AND(ALUOperand::D8)),
            0xe7 => Ok(Opcode::RST(RSTAddress::X20)),
            0xe8 => Ok(Opcode::ADDSP),
            0xe9 => Ok(Opcode::JPI),
            0xea => Ok(Opcode::LD(LDType::IndirectFromA(Indirect::WordIndirect))),
            0xee => Ok(Opcode::XOR(ALUOperand::D8)),
            0xef => Ok(Opcode::RST(RSTAddress::X28)),
            0xf0 => Ok(Opcode::LD(LDType::AddressFromA)),
            0xf1 => Ok(Opcode::POP(StackTarget::AF)),
            0xf2 => Ok(Opcode::LD(LDType::AFromIndirect(Indirect::LastByteIndirect))),
            0xf3 => Ok(Opcode::DI),
            0xf5 => Ok(Opcode::PUSH(StackTarget::AF)),
            0xf6 => Ok(Opcode::OR(ALUOperand::D8)),
            0xf7 => Ok(Opcode::RST(RSTAddress::X30)),
            0xf8 => Ok(Opcode::LD(LDType::IndirectFromSP)),
            0xf9 => Ok(Opcode::LD(LDType::SPFromHL)),
            0xfa => Ok(Opcode::LD(LDType::AFromIndirect(Indirect::WordIndirect))),
            0xfb => Ok(Opcode::EI),
            0xfe => Ok(Opcode::CP(ALUOperand::D8)),
            0xff => Ok(Opcode::RST(RSTAddress::X38)),
            _ => Err("unknown opcode"),
        }
    }
}


impl TryFrom<u8> for PrefixedOpcode {
    type Error = &'static str;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(PrefixedOpcode::RLC(RotateOperand::B)),
            0x01 => Ok(PrefixedOpcode::RLC(RotateOperand::C)),
            0x02 => Ok(PrefixedOpcode::RLC(RotateOperand::D)),
            0x03 => Ok(PrefixedOpcode::RLC(RotateOperand::E)),
            0x04 => Ok(PrefixedOpcode::RLC(RotateOperand::H)),
            0x05 => Ok(PrefixedOpcode::RLC(RotateOperand::L)),
            0x06 => Ok(PrefixedOpcode::RLC(RotateOperand::HLIndirect)),
            0x07 => Ok(PrefixedOpcode::RLC(RotateOperand::A)),
            _ => Err("unknown prefixed opcode"),
        }
    }
}

impl CPU {
    pub fn new() -> CPU {
        let mut cpu = CPU {
            reg: Registers::new(),
            counter: 20,
            memory_bus: MemoryBus::new(),
            // TODO remove
            tmp_buffer: vec![1; 100],
            IME: true,
            running: true,
        };

        // TODO error handling

        // FIXME pass this from main
        let f = File::open("/home/iaguis/programming/gameboy/cpu_instrs/cpu_instrs.gb").expect("can't open ROM");
        cpu.memory_bus.read_rom(f).expect("can't read ROM");

        cpu
    }

    fn fetch_byte(&mut self) -> Result<Opcode, &'static str> {
        println!("pc = {:#04x}", self.reg.pc);
        let b = self.memory_bus.read_byte(self.reg.pc.into());
        println!("mem[pc] = {:#04x}", b);

        let opcode = Opcode::try_from(b)?;
        Ok(opcode)
    }

    fn fetch_prefixed_byte(&mut self) -> Result<PrefixedOpcode, &'static str> {
        println!("pc = {:#04x}", self.reg.pc);
        let b = self.memory_bus.read_byte(self.reg.pc.into());
        println!("mem[pc] = {:#04x}", b);

        let prefixed_opcode = PrefixedOpcode::try_from(b)?;
        Ok(prefixed_opcode)
    }

    // TODO double-check cycles
    fn execute(&mut self) -> u8 {
        // XXX this panics if it fails to decode the opcode, which is probably fine
        let opcode = self.fetch_byte().expect("failed fetching");

        let mut cycles = 1;

        println!("opcode: {:?}", opcode);
        match opcode {
            Opcode::NOP => {
                println!("nop, sleeping 1s");
                cycles = 1;
                self.reg.pc += 1;
            },

            Opcode::LD(ld_type) => {
                match ld_type {
                    LDType::Byte(target, source) => {
                        let source_val = match source {
                            LDSource::A => self.reg.a,
                            LDSource::B => self.reg.b,
                            LDSource::C => self.reg.c,
                            LDSource::D => self.reg.d,
                            LDSource::E => self.reg.e,
                            LDSource::H => self.reg.h,
                            LDSource::L => self.reg.l,
                            LDSource::D8 => self.memory_bus.read_byte(self.reg.pc + 1),
                            LDSource::HLIndirect => self.memory_bus.read_byte(self.reg.hl()),
                        };
                        match target {
                            LDTarget::A => self.reg.a = source_val,
                            LDTarget::B => self.reg.b = source_val,
                            LDTarget::C => self.reg.c = source_val,
                            LDTarget::D => self.reg.d = source_val,
                            LDTarget::E => self.reg.e = source_val,
                            LDTarget::H => self.reg.h = source_val,
                            LDTarget::L => self.reg.l = source_val,
                            LDTarget::HLIndirect => {
                                self.memory_bus.write_byte(self.reg.hl(), source_val)
                            }
                        }

                        match source {
                            LDSource::D8 => {cycles = 2; self.reg.pc += 2},
                            LDSource::HLIndirect => {cycles = 1; self.reg.pc += 2 },
                            _ => {cycles = 1; self.reg.pc += 1}
                        }
                    },
                    LDType::Word(ld_word_target) => {
                        // little-endian
                        let msb = self.memory_bus.read_byte(self.reg.pc + 2);
                        let lsb = self.memory_bus.read_byte(self.reg.pc + 1);
                        match ld_word_target {
                            LDWordTarget::BC => {
                                self.reg.b = msb;
                                self.reg.c = lsb;
                            },
                            LDWordTarget::DE => {
                                self.reg.d = msb;
                                self.reg.e = lsb;
                            },
                            LDWordTarget::HL => {
                                self.reg.h = msb;
                                self.reg.l = lsb;
                            },
                            LDWordTarget::SP => {
                                self.reg.sp = ((msb as u16) << 8) | lsb as u16
                            },
                        }

                        cycles = 3;
                        self.reg.pc += 3;
                    },
                    LDType::IndirectFromA(indirect) => {
                        let a = self.reg.a;

                        match indirect {
                            Indirect::BCIndirect => {
                                let bc = self.reg.bc();
                                self.memory_bus.write_byte(bc, a);
                            }
                            Indirect::DEIndirect => {
                                let de = self.reg.de();
                                self.memory_bus.write_byte(de, a);
                            }
                            Indirect::HLIndirectInc => {
                                let r = self.reg.alu_inc16(self.reg.hl());
                                self.reg.set_hl(r);

                                self.memory_bus.write_byte(r, a);
                            }
                            Indirect::HLIndirectDec => {
                                let r = self.reg.alu_dec16(self.reg.hl());
                                self.reg.set_hl(r);

                                self.memory_bus.write_byte(r, a);
                            }
                            Indirect::WordIndirect => {
                                let msb = self.memory_bus.read_byte(self.reg.pc + 2);
                                let lsb = self.memory_bus.read_byte(self.reg.pc + 1);
                                let address = ((msb as u16) << 8) | lsb as u16;

                                self.memory_bus.write_byte(address, a);

                            }
                            Indirect::LastByteIndirect => {
                                let c = self.reg.c as u16;
                                self.memory_bus.write_byte(0xFF00 + c, a);
                            }
                        }
                        match indirect {
                            Indirect::WordIndirect => {
                                cycles = 4;
                                self.reg.pc += 3;
                            },
                            _ => {
                                cycles = 2;
                                self.reg.pc += 1;
                            }
                        }
                    },
                    LDType::AddressFromA => {
                        let msb = self.memory_bus.read_byte(self.reg.pc + 2);
                        let lsb = self.memory_bus.read_byte(self.reg.pc + 1);
                        let address = ((msb as u16) << 8) | lsb as u16;

                        println!("address {:#04x}", address);

                        self.memory_bus.write_byte(address, self.reg.a);

                        cycles = 4;
                        self.reg.pc += 3;
                    },

                    LDType::AFromAddress => {
                        let msb = self.memory_bus.read_byte(self.reg.pc + 2);
                        let lsb = self.memory_bus.read_byte(self.reg.pc + 1);
                        let address = ((msb as u16) << 8) | lsb as u16;

                        println!("address {:#04x}", address);

                        self.reg.a = self.memory_bus.read_byte(address);

                        cycles = 4;
                        self.reg.pc += 3;
                    },

                    LDType::AFromIndirect(indirect) => {
                        match indirect {
                            Indirect::BCIndirect => {
                                let bc = self.reg.bc();
                                self.reg.a = self.memory_bus.read_byte(bc);
                            }
                            Indirect::DEIndirect => {
                                let de = self.reg.de();
                                self.reg.a = self.memory_bus.read_byte(de);
                            }
                            Indirect::HLIndirectInc => {
                                let r = self.reg.alu_inc16(self.reg.hl());
                                self.reg.set_hl(r);

                                self.reg.a = self.memory_bus.read_byte(r);
                            }
                            Indirect::HLIndirectDec => {
                                let r = self.reg.alu_dec16(self.reg.hl());
                                self.reg.set_hl(r);

                                self.reg.a = self.memory_bus.read_byte(r);
                            }
                            Indirect::WordIndirect => {
                                let msb = self.memory_bus.read_byte(self.reg.pc + 2);
                                let lsb = self.memory_bus.read_byte(self.reg.pc + 1);
                                let address = ((msb as u16) << 8) | lsb as u16;

                                self.reg.a = self.memory_bus.read_byte(address);
                            }
                            Indirect::LastByteIndirect => {
                                let c = self.reg.c as u16;
                                self.reg.a = self.memory_bus.read_byte(0xFF00 + c);
                            }
                        }
                        match indirect {
                            Indirect::WordIndirect => {
                                cycles = 4;
                                self.reg.pc += 3;
                            },
                            _ => {
                                cycles = 2;
                                self.reg.pc += 1;
                            }
                        }
                    },

                    LDType::SPFromHL => {
                        self.reg.sp = self.reg.hl();

                        cycles = 2;
                        self.reg.pc += 1;
                    },

                    LDType::IndirectFromSP => {
                        let msb = self.memory_bus.read_byte(self.reg.pc + 2);
                        let lsb = self.memory_bus.read_byte(self.reg.pc + 1);
                        let address = ((msb as u16) << 8) | lsb as u16;

                        self.memory_bus.write_byte(address, (self.reg.sp & 0xff) as u8);
                        self.memory_bus.write_byte(address+1, (self.reg.sp >> 8) as u8);

                        cycles = 5;
                        self.reg.pc += 3;
                    },
                }
            },

            Opcode::INC(target) => {
                match target {
                    IncDecTarget::A => { self.reg.a = self.reg.alu_inc(self.reg.a); },
                    IncDecTarget::B => { self.reg.b = self.reg.alu_inc(self.reg.b); },
                    IncDecTarget::C => { self.reg.c = self.reg.alu_inc(self.reg.c); },
                    IncDecTarget::D => { self.reg.d = self.reg.alu_inc(self.reg.d); },
                    IncDecTarget::E => { self.reg.e = self.reg.alu_inc(self.reg.e); },
                    IncDecTarget::H => { self.reg.h = self.reg.alu_inc(self.reg.h); },
                    IncDecTarget::L => { self.reg.l = self.reg.alu_inc(self.reg.l); },
                    IncDecTarget::BC => {
                        let r = self.reg.alu_inc16(self.reg.bc());
                        self.reg.set_bc(r);
                    },
                    IncDecTarget::DE => {
                        let r = self.reg.alu_inc16(self.reg.de());
                        self.reg.set_de(r);
                    },
                    IncDecTarget::HL => {
                        let r = self.reg.alu_inc16(self.reg.hl());
                        self.reg.set_hl(r);
                    },
                    IncDecTarget::SP => {
                        let r = self.reg.alu_inc16(self.reg.sp);
                        self.reg.sp = r;
                    },
                    IncDecTarget::HLIndirect => {
                        let val = self.memory_bus.read_byte(self.reg.hl());
                        let r = self.reg.alu_dec(val);
                        self.memory_bus.write_byte(self.reg.hl(), r);
                    },
                }

                match target {
                    IncDecTarget::HLIndirect => { cycles = 3; },
                    IncDecTarget::BC | IncDecTarget::DE | IncDecTarget::HL | IncDecTarget::SP => { cycles = 2; },
                    _ => { cycles = 1; },
                }
                self.reg.pc += 1;
            },

            Opcode::DEC(target) => {
                match target {
                    IncDecTarget::A => { self.reg.a = self.reg.alu_dec(self.reg.a); },
                    IncDecTarget::B => { self.reg.b = self.reg.alu_dec(self.reg.b); },
                    IncDecTarget::C => { self.reg.c = self.reg.alu_dec(self.reg.c); },
                    IncDecTarget::D => { self.reg.d = self.reg.alu_dec(self.reg.d); },
                    IncDecTarget::E => { self.reg.e = self.reg.alu_dec(self.reg.e); },
                    IncDecTarget::H => { self.reg.h = self.reg.alu_dec(self.reg.h); },
                    IncDecTarget::L => { self.reg.l = self.reg.alu_dec(self.reg.l); },
                    IncDecTarget::BC => {
                        let r = self.reg.alu_dec16(self.reg.bc());
                        self.reg.set_bc(r);
                    },
                    IncDecTarget::DE => {
                        let r = self.reg.alu_dec16(self.reg.de());
                        self.reg.set_de(r);
                    },
                    IncDecTarget::HL => {
                        let r = self.reg.alu_dec16(self.reg.hl());
                        self.reg.set_hl(r);
                    },
                    IncDecTarget::SP => {
                        let r = self.reg.alu_dec16(self.reg.sp);
                        self.reg.sp = r;
                    },
                    IncDecTarget::HLIndirect => {
                        let val = self.memory_bus.read_byte(self.reg.hl());
                        let r = self.reg.alu_dec(val);
                        self.memory_bus.write_byte(self.reg.hl(), r);
                    },
                }

                match target {
                    IncDecTarget::HLIndirect => { cycles = 3; },
                    IncDecTarget::BC | IncDecTarget::DE | IncDecTarget::HL | IncDecTarget::SP => { cycles = 2; },
                    _ => { cycles = 1 },
                }
                self.reg.pc += 1;
            },

            Opcode::ADD(operand) => {
                match operand {
                    ALUOperand::A => { self.reg.alu_add(self.reg.a) },
                    ALUOperand::B => { self.reg.alu_add(self.reg.b) },
                    ALUOperand::C => { self.reg.alu_add(self.reg.c) },
                    ALUOperand::D => { self.reg.alu_add(self.reg.d) },
                    ALUOperand::E => { self.reg.alu_add(self.reg.e) },
                    ALUOperand::H => { self.reg.alu_add(self.reg.h) },
                    ALUOperand::L => { self.reg.alu_add(self.reg.l) },
                    ALUOperand::HLIndirect => {
                        let data = self.memory_bus.read_byte(self.reg.hl());
                        self.reg.alu_add(data);
                    },
                    ALUOperand::D8 => {
                        let data = self.memory_bus.read_byte(self.reg.pc + 1);
                        self.reg.alu_add(data);
                    },
                }
                cycles = 1;
                self.reg.pc += 1;
            },

            Opcode::ADDHL(operand) => {
                match operand {
                    ADDHLOperand::BC => {
                        self.reg.alu_addhl(self.reg.bc());
                    },
                    ADDHLOperand::DE => {
                        self.reg.alu_addhl(self.reg.de());
                    },
                    ADDHLOperand::HL => {
                        self.reg.alu_addhl(self.reg.hl());
                    },
                    ADDHLOperand::SP => {
                        self.reg.alu_addhl(self.reg.sp);
                    },
                }
            },

            Opcode::ADDSP => {
                let val = self.memory_bus.read_byte(self.reg.pc + 1);
                self.reg.alu_addsp(val);
            },

            Opcode::ADC(operand) => {
                match operand {
                    ALUOperand::A => { self.reg.alu_adc(self.reg.a) },
                    ALUOperand::B => { self.reg.alu_adc(self.reg.b) },
                    ALUOperand::C => { self.reg.alu_adc(self.reg.c) },
                    ALUOperand::D => { self.reg.alu_adc(self.reg.d) },
                    ALUOperand::E => { self.reg.alu_adc(self.reg.e) },
                    ALUOperand::H => { self.reg.alu_adc(self.reg.h) },
                    ALUOperand::L => { self.reg.alu_adc(self.reg.l) },
                    ALUOperand::HLIndirect => {
                        let data = self.memory_bus.read_byte(self.reg.hl());
                        self.reg.alu_adc(data);
                    },
                    ALUOperand::D8 => {
                        let data = self.memory_bus.read_byte(self.reg.pc + 1);
                        self.reg.alu_adc(data);
                    },
                }
                cycles = 1;
                self.reg.pc += 1;
            },

            Opcode::SUB(operand) => {
                match operand {
                    ALUOperand::A => { self.reg.alu_sub(self.reg.a) },
                    ALUOperand::B => { self.reg.alu_sub(self.reg.b) },
                    ALUOperand::C => { self.reg.alu_sub(self.reg.c) },
                    ALUOperand::D => { self.reg.alu_sub(self.reg.d) },
                    ALUOperand::E => { self.reg.alu_sub(self.reg.e) },
                    ALUOperand::H => { self.reg.alu_sub(self.reg.h) },
                    ALUOperand::L => { self.reg.alu_sub(self.reg.l) },
                    ALUOperand::HLIndirect => {
                        let data = self.memory_bus.read_byte(self.reg.hl());
                        self.reg.alu_sub(data);
                    },
                    ALUOperand::D8 => {
                        let data = self.memory_bus.read_byte(self.reg.pc + 1);
                        self.reg.alu_sub(data);
                    },
                }
                cycles = 1;
                self.reg.pc += 1;
            }

            Opcode::SBC(operand) => {
                match operand {
                    ALUOperand::A => { self.reg.alu_sbc(self.reg.a) },
                    ALUOperand::B => { self.reg.alu_sbc(self.reg.b) },
                    ALUOperand::C => { self.reg.alu_sbc(self.reg.c) },
                    ALUOperand::D => { self.reg.alu_sbc(self.reg.d) },
                    ALUOperand::E => { self.reg.alu_sbc(self.reg.e) },
                    ALUOperand::H => { self.reg.alu_sbc(self.reg.h) },
                    ALUOperand::L => { self.reg.alu_sbc(self.reg.l) },
                    ALUOperand::HLIndirect => {
                        let data = self.memory_bus.read_byte(self.reg.hl());
                        self.reg.alu_sbc(data);
                    },
                    ALUOperand::D8 => {
                        let data = self.memory_bus.read_byte(self.reg.pc + 1);
                        self.reg.alu_sbc(data);
                    },
                }
                cycles = 1;
                self.reg.pc += 1;
            }

            Opcode::AND(operand) => {
                match operand {
                    ALUOperand::A => { self.reg.alu_and(self.reg.a) },
                    ALUOperand::B => { self.reg.alu_and(self.reg.b) },
                    ALUOperand::C => { self.reg.alu_and(self.reg.c) },
                    ALUOperand::D => { self.reg.alu_and(self.reg.d) },
                    ALUOperand::E => { self.reg.alu_and(self.reg.e) },
                    ALUOperand::H => { self.reg.alu_and(self.reg.h) },
                    ALUOperand::L => { self.reg.alu_and(self.reg.l) },
                    ALUOperand::HLIndirect => {
                        let data = self.memory_bus.read_byte(self.reg.hl());
                        self.reg.alu_and(data);
                    },
                    ALUOperand::D8 => {
                        let data = self.memory_bus.read_byte(self.reg.pc + 1);
                        self.reg.alu_and(data);
                    },
                }
                cycles = 1;
                self.reg.pc += 1;
            }

            Opcode::XOR(operand) => {
                match operand {
                    ALUOperand::A => { self.reg.alu_xor(self.reg.a) },
                    ALUOperand::B => { self.reg.alu_xor(self.reg.b) },
                    ALUOperand::C => { self.reg.alu_xor(self.reg.c) },
                    ALUOperand::D => { self.reg.alu_xor(self.reg.d) },
                    ALUOperand::E => { self.reg.alu_xor(self.reg.e) },
                    ALUOperand::H => { self.reg.alu_xor(self.reg.h) },
                    ALUOperand::L => { self.reg.alu_xor(self.reg.l) },
                    ALUOperand::HLIndirect => {
                        let data = self.memory_bus.read_byte(self.reg.hl());
                        self.reg.alu_xor(data);
                    },
                    ALUOperand::D8 => {
                        let data = self.memory_bus.read_byte(self.reg.pc + 1);
                        self.reg.alu_xor(data);
                    },
                }
                cycles = 1;
                self.reg.pc += 1;
            }

            Opcode::OR(operand) => {
                match operand {
                    ALUOperand::A => { self.reg.alu_or(self.reg.a) },
                    ALUOperand::B => { self.reg.alu_or(self.reg.b) },
                    ALUOperand::C => { self.reg.alu_or(self.reg.c) },
                    ALUOperand::D => { self.reg.alu_or(self.reg.d) },
                    ALUOperand::E => { self.reg.alu_or(self.reg.e) },
                    ALUOperand::H => { self.reg.alu_or(self.reg.h) },
                    ALUOperand::L => { self.reg.alu_or(self.reg.l) },
                    ALUOperand::HLIndirect => {
                        let data = self.memory_bus.read_byte(self.reg.hl());
                        self.reg.alu_or(data);
                    },
                    ALUOperand::D8 => {
                        let data = self.memory_bus.read_byte(self.reg.pc + 1);
                        self.reg.alu_or(data);
                    },
                }
                cycles = 1;
                self.reg.pc += 1;
            }

            Opcode::CP(operand) => {
                match operand {
                    ALUOperand::A => { self.reg.alu_cp(self.reg.a) },
                    ALUOperand::B => { self.reg.alu_cp(self.reg.b) },
                    ALUOperand::C => { self.reg.alu_cp(self.reg.c) },
                    ALUOperand::D => { self.reg.alu_cp(self.reg.d) },
                    ALUOperand::E => { self.reg.alu_cp(self.reg.e) },
                    ALUOperand::H => { self.reg.alu_cp(self.reg.h) },
                    ALUOperand::L => { self.reg.alu_cp(self.reg.l) },
                    ALUOperand::HLIndirect => {
                        let data = self.memory_bus.read_byte(self.reg.hl());
                        self.reg.alu_cp(data);
                    },
                    ALUOperand::D8 => {
                        let data = self.memory_bus.read_byte(self.reg.pc + 1);
                        self.reg.alu_cp(data);
                    },
                }
                cycles = 1;
                self.reg.pc += 1;
            }

            Opcode::RLCA => {
                let r = (self.reg.a << 1) | (self.reg.a >> 7);

                cycles = 1;
                self.reg.a = r;
                self.reg.pc += 1;
            },

            Opcode::RLA => {
                let c = if self.reg.get_flag(Flag::C) { 1 } else { 0 };
                let r = self.reg.a << 1 | c;
                self.reg.set_flag(Flag::Z, r == 0);
                self.reg.set_flag(Flag::N, false);
                self.reg.set_flag(Flag::H, false);
                self.reg.set_flag(Flag::C, (0x80 & self.reg.a) == 0x80);

                cycles = 1;
                self.reg.a = r;
                self.reg.pc += 1;
            },

            Opcode::RRCA => {
                let r = (self.reg.a >> 1) | (self.reg.a << 7);

                cycles = 1;
                self.reg.a = r;
                self.reg.pc += 1;
            },

            Opcode::RRA => {
                let c = if self.reg.get_flag(Flag::C) { 1 } else { 0 } << 7;
                let r = c | self.reg.a >> 1;
                self.reg.set_flag(Flag::Z, r == 0);
                self.reg.set_flag(Flag::N, false);
                self.reg.set_flag(Flag::H, false);
                self.reg.set_flag(Flag::C, (0x01 & self.reg.a) == 0x01);

                cycles = 1;
                self.reg.a = r;
                self.reg.pc += 1;
            },

            Opcode::CPL => { self.reg.alu_cpl(); cycles = 1; self.reg.pc += 1; },

            Opcode::CCF => { self.reg.alu_ccf(); cycles = 1; self.reg.pc += 1; },

            Opcode::SCF => { self.reg.alu_scf(); cycles = 1; self.reg.pc += 1; },

            Opcode::DAA => todo!(),

            Opcode::JP(condition) => {
                let msb = self.memory_bus.read_byte(self.reg.pc + 2);
                let lsb = self.memory_bus.read_byte(self.reg.pc + 1);

                let jp_address = ((msb as u16) << 8) | (lsb as u16);

                match condition {
                    JCondition::Nothing => {
                        self.reg.pc = jp_address;
                        cycles = 4;
                    },
                    JCondition::NZ => {
                        if !self.reg.get_flag(Flag::Z) {
                            self.reg.pc = jp_address;
                            cycles = 4;
                        } else {
                            self.reg.pc += 1;
                            cycles = 3;
                        }
                    },
                    JCondition::NC => {
                        if !self.reg.get_flag(Flag::C) {
                            self.reg.pc = jp_address;
                            cycles = 4;
                        } else {
                            self.reg.pc += 1;
                            cycles = 3;
                        }
                    },
                    JCondition::Z => {
                        if self.reg.get_flag(Flag::Z) {
                            self.reg.pc = jp_address;
                            cycles = 4;
                        } else {
                            self.reg.pc += 1;
                            cycles = 3;
                        }
                    },
                    JCondition::C => {
                        if self.reg.get_flag(Flag::C) {
                            self.reg.pc = jp_address;
                            cycles = 4;
                        } else {
                            self.reg.pc += 1;
                            cycles = 3;
                        }
                    },
                }
            },

            Opcode::JPI => {
                cycles = 1;
                self.reg.pc = self.reg.hl();
            },

            Opcode::JR(condition) => {
                let offset = self.memory_bus.read_byte(self.reg.pc + 1) as i8;
                self.reg.pc += 1;

                let jp_address = (self.reg.pc + 1).wrapping_add(offset as u16);

                match condition {
                    JCondition::Nothing => {
                        self.reg.pc = jp_address;
                        cycles = 4;
                    },
                    JCondition::NZ => {
                        if !self.reg.get_flag(Flag::Z) {
                            self.reg.pc = jp_address;
                            cycles = 4;
                        } else {
                            self.reg.pc += 1;
                            cycles = 3;
                        }
                    },
                    JCondition::NC => {
                        if !self.reg.get_flag(Flag::C) {
                            self.reg.pc = jp_address;
                            cycles = 4;
                        } else {
                            self.reg.pc += 1;
                            cycles = 3;
                        }
                    },
                    JCondition::Z => {
                        if self.reg.get_flag(Flag::Z) {
                            self.reg.pc = jp_address;
                            cycles = 4;
                        } else {
                            self.reg.pc += 1;
                            cycles = 3;
                        }
                    },
                    JCondition::C => {
                        if self.reg.get_flag(Flag::C) {
                            self.reg.pc = jp_address;
                            cycles = 4;
                        } else {
                            self.reg.pc += 1;
                            cycles = 3;
                        }
                    },
                }
            },

            // TODO refactor
            Opcode::CALL(condition) => {
                match condition  {
                    JCondition::Nothing => {
                        let msb = self.memory_bus.read_byte(self.reg.pc + 2);
                        let lsb = self.memory_bus.read_byte(self.reg.pc + 1);
                        self.reg.sp -= 1;
                        self.memory_bus.write_byte(self.reg.sp, (self.reg.pc >> 8) as u8);
                        self.memory_bus.write_byte(self.reg.sp, (self.reg.pc & 0xFF) as u8);

                        self.reg.pc = ((msb as u16) << 8) | ((lsb as u16) & 0xFF);
                        cycles = 6;
                    },
                    JCondition::NZ => {
                        let msb = self.memory_bus.read_byte(self.reg.pc + 2);
                        let lsb = self.memory_bus.read_byte(self.reg.pc + 1);

                        if !self.reg.get_flag(Flag::Z) {
                            self.reg.sp -= 1;
                            self.memory_bus.write_byte(self.reg.sp, (self.reg.pc >> 8) as u8);
                            self.memory_bus.write_byte(self.reg.sp, (self.reg.pc & 0xFF) as u8);

                            self.reg.pc = ((msb as u16) << 8) | ((lsb as u16) & 0xFF);
                            cycles = 6;
                        } else {
                            cycles = 3;
                            self.reg.pc += 1;
                        }
                    },
                    JCondition::NC => {
                        let msb = self.memory_bus.read_byte(self.reg.pc + 2);
                        let lsb = self.memory_bus.read_byte(self.reg.pc + 1);

                        if !self.reg.get_flag(Flag::C) {
                            self.reg.sp -= 1;
                            self.memory_bus.write_byte(self.reg.sp, (self.reg.pc >> 8) as u8);
                            self.memory_bus.write_byte(self.reg.sp, (self.reg.pc & 0xFF) as u8);

                            self.reg.pc = ((msb as u16) << 8) | ((lsb as u16) & 0xFF);
                            cycles = 6;
                        } else {
                            cycles = 3;
                            self.reg.pc += 1;
                        }
                    },
                    JCondition::Z => {
                        let msb = self.memory_bus.read_byte(self.reg.pc + 2);
                        let lsb = self.memory_bus.read_byte(self.reg.pc + 1);

                        if self.reg.get_flag(Flag::Z) {
                            self.reg.sp -= 1;
                            self.memory_bus.write_byte(self.reg.sp, (self.reg.pc >> 8) as u8);
                            self.memory_bus.write_byte(self.reg.sp, (self.reg.pc & 0xFF) as u8);

                            self.reg.pc = ((msb as u16) << 8) | ((lsb as u16) & 0xFF);
                            cycles = 6;
                        } else {
                            cycles = 3;
                            self.reg.pc += 1;
                        }
                    },
                    JCondition::C => {
                        let msb = self.memory_bus.read_byte(self.reg.pc + 2);
                        let lsb = self.memory_bus.read_byte(self.reg.pc + 1);

                        if self.reg.get_flag(Flag::C) {
                            self.reg.sp -= 1;
                            self.memory_bus.write_byte(self.reg.sp, (self.reg.pc >> 8) as u8);
                            self.memory_bus.write_byte(self.reg.sp, (self.reg.pc & 0xFF) as u8);

                            self.reg.pc = ((msb as u16) << 8) | ((lsb as u16) & 0xFF);
                            cycles = 6;
                        } else {
                            cycles = 3;
                            self.reg.pc += 1;
                        }
                    },
                }
            }

            // TODO refactor
            Opcode::RET(condition) => {
                match condition {
                    JCondition::Nothing => {
                        let msb = self.memory_bus.read_byte(self.reg.sp + 2);
                        let lsb = self.memory_bus.read_byte(self.reg.sp + 1);

                        cycles = 4;
                        self.reg.pc = ((msb as u16) << 8) | ((lsb as u16) & 0xFF);
                        self.reg.sp += 2;
                    },
                    JCondition::NZ => {
                        if !self.reg.get_flag(Flag::Z) {
                            let msb = self.memory_bus.read_byte(self.reg.sp + 2);
                            let lsb = self.memory_bus.read_byte(self.reg.sp + 1);

                            cycles = 5;
                            self.reg.pc = ((msb as u16) << 8) | ((lsb as u16) & 0xFF);
                            self.reg.sp += 2;
                        } else {
                            cycles = 2;
                            self.reg.pc += 1;
                        }
                    }
                    JCondition::NC => {
                        if !self.reg.get_flag(Flag::C) {
                            let msb = self.memory_bus.read_byte(self.reg.sp + 2);
                            let lsb = self.memory_bus.read_byte(self.reg.sp + 1);

                            cycles = 5;
                            self.reg.pc = ((msb as u16) << 8) | ((lsb as u16) & 0xFF);
                            self.reg.sp += 2;
                        } else {
                            cycles = 2;
                            self.reg.pc += 1;
                        }
                    }
                    JCondition::Z => {
                        if self.reg.get_flag(Flag::Z) {
                            let msb = self.memory_bus.read_byte(self.reg.sp + 2);
                            let lsb = self.memory_bus.read_byte(self.reg.sp + 1);

                            cycles = 5;
                            self.reg.pc = ((msb as u16) << 8) | ((lsb as u16) & 0xFF);
                            self.reg.sp += 2;
                        } else {
                            cycles = 2;
                            self.reg.pc += 1;
                        }
                    }
                    JCondition::C => {
                        if self.reg.get_flag(Flag::C) {
                            let msb = self.memory_bus.read_byte(self.reg.sp + 2);
                            let lsb = self.memory_bus.read_byte(self.reg.sp + 1);

                            cycles = 5;
                            self.reg.pc = ((msb as u16) << 8) | ((lsb as u16) & 0xFF);
                            self.reg.sp += 2;
                        } else {
                            cycles = 2;
                            self.reg.pc += 1;
                        }
                    }
                }
            }

            Opcode::RETI => {
                let msb = self.memory_bus.read_byte(self.reg.sp + 2);
                let lsb = self.memory_bus.read_byte(self.reg.sp + 1);

                cycles = 4;
                self.reg.pc = ((msb as u16) << 8) | ((lsb as u16) & 0xFF);
                self.reg.sp += 2;
                self.IME = true;
            },

            Opcode::RST(address) => {
                match address {
                    RSTAddress::X00 => {
                        let n = 0x0000;
                        self.reg.sp -= 1;
                        self.memory_bus.write_byte(self.reg.sp, (self.reg.pc >> 8) as u8);
                        self.reg.sp -= 1;
                        self.memory_bus.write_byte(self.reg.sp, (self.reg.pc & 0xFF) as u8);

                        cycles = 4;
                        self.reg.pc = n;
                    }
                    RSTAddress::X10 => {
                        let n = 0x0010;
                        self.reg.sp -= 1;
                        self.memory_bus.write_byte(self.reg.sp, (self.reg.pc >> 8) as u8);
                        self.reg.sp -= 1;
                        self.memory_bus.write_byte(self.reg.sp, (self.reg.pc & 0xFF) as u8);

                        cycles = 4;
                        self.reg.pc = n;
                    }
                    RSTAddress::X20 => {
                        let n = 0x0020;
                        self.reg.sp -= 1;
                        self.memory_bus.write_byte(self.reg.sp, (self.reg.pc >> 8) as u8);
                        self.reg.sp -= 1;
                        self.memory_bus.write_byte(self.reg.sp, (self.reg.pc & 0xFF) as u8);

                        cycles = 4;
                        self.reg.pc = n;
                    }
                    RSTAddress::X30 => {
                        let n = 0x0030;
                        self.reg.sp -= 1;
                        self.memory_bus.write_byte(self.reg.sp, (self.reg.pc >> 8) as u8);
                        self.reg.sp -= 1;
                        self.memory_bus.write_byte(self.reg.sp, (self.reg.pc & 0xFF) as u8);

                        cycles = 4;
                        self.reg.pc = n;
                    }
                    RSTAddress::X08 => {
                        let n = 0x0008;
                        self.reg.sp -= 1;
                        self.memory_bus.write_byte(self.reg.sp, (self.reg.pc >> 8) as u8);
                        self.reg.sp -= 1;
                        self.memory_bus.write_byte(self.reg.sp, (self.reg.pc & 0xFF) as u8);

                        cycles = 4;
                        self.reg.pc = n;
                    }
                    RSTAddress::X18 => {
                        let n = 0x0018;
                        self.reg.sp -= 1;
                        self.memory_bus.write_byte(self.reg.sp, (self.reg.pc >> 8) as u8);
                        self.reg.sp -= 1;
                        self.memory_bus.write_byte(self.reg.sp, (self.reg.pc & 0xFF) as u8);

                        cycles = 4;
                        self.reg.pc = n;
                    }
                    RSTAddress::X28 => {
                        let n = 0x0028;
                        self.reg.sp -= 1;
                        self.memory_bus.write_byte(self.reg.sp, (self.reg.pc >> 8) as u8);
                        self.reg.sp -= 1;
                        self.memory_bus.write_byte(self.reg.sp, (self.reg.pc & 0xFF) as u8);

                        cycles = 4;
                        self.reg.pc = n;
                    }
                    RSTAddress::X38 => {
                        let n = 0x0038;
                        self.reg.sp -= 1;
                        self.memory_bus.write_byte(self.reg.sp, (self.reg.pc >> 8) as u8);
                        self.reg.sp -= 1;
                        self.memory_bus.write_byte(self.reg.sp, (self.reg.pc & 0xFF) as u8);

                        cycles = 4;
                        self.reg.pc = n;
                    }
                }
            },

            Opcode::HALT => {
                // TODO handle interrupt state
                self.running = false;
            },

            Opcode::PUSH(target) => {
                self.reg.sp -= 1;

                match target {
                    StackTarget::AF => {
                        self.memory_bus.write_byte(self.reg.sp, self.reg.a);
                        self.reg.sp -= 1;
                        self.memory_bus.write_byte(self.reg.sp, self.reg.f);
                    },
                    StackTarget::BC => {
                        self.memory_bus.write_byte(self.reg.sp, self.reg.b);
                        self.reg.sp -= 1;
                        self.memory_bus.write_byte(self.reg.sp, self.reg.c);
                    },
                    StackTarget::DE => {
                        self.memory_bus.write_byte(self.reg.sp, self.reg.d);
                        self.reg.sp -= 1;
                        self.memory_bus.write_byte(self.reg.sp, self.reg.e);
                    },
                    StackTarget::HL => {
                        self.memory_bus.write_byte(self.reg.sp, self.reg.h);
                        self.reg.sp -= 1;
                        self.memory_bus.write_byte(self.reg.sp, self.reg.l);
                    },
                }

                cycles = 4;
                self.reg.pc += 1;
            },

            Opcode::POP(target) => {
                match target {
                    StackTarget::AF => {
                        let lsb = self.memory_bus.read_byte(self.reg.sp);
                        self.reg.sp += 1;
                        let msb = self.memory_bus.read_byte(self.reg.sp);
                        self.reg.sp += 1;
                        self.reg.a = msb;
                        self.reg.f = lsb;
                    },
                    StackTarget::BC => {
                        let lsb = self.memory_bus.read_byte(self.reg.sp);
                        self.reg.sp += 1;
                        let msb = self.memory_bus.read_byte(self.reg.sp);
                        self.reg.sp += 1;
                        self.reg.a = msb;
                        self.reg.f = lsb;
                    },
                    StackTarget::DE => {
                        let lsb = self.memory_bus.read_byte(self.reg.sp);
                        self.reg.sp += 1;
                        let msb = self.memory_bus.read_byte(self.reg.sp);
                        self.reg.sp += 1;
                        self.reg.a = msb;
                        self.reg.f = lsb;
                    },
                    StackTarget::HL => {
                        let lsb = self.memory_bus.read_byte(self.reg.sp);
                        self.reg.sp += 1;
                        let msb = self.memory_bus.read_byte(self.reg.sp);
                        self.reg.sp += 1;
                        self.reg.a = msb;
                        self.reg.f = lsb;
                    },
                }

                cycles = 3;
                self.reg.pc += 1;
            },

            Opcode::DI => {
                self.IME = false;

                cycles = 1;
                self.reg.pc += 1;
            },

            Opcode::EI => {
                self.IME = true;

                cycles = 1;
                self.reg.pc += 1;
            },

            Opcode::PREFIX => {
                let prefixed_opcode = self.fetch_prefixed_byte().expect("failed fetching");

                match prefixed_opcode {
                    PrefixedOpcode::RLC(operand) => {
                        match operand {
                            RotateOperand::A => {
                                todo!();
                            },
                            RotateOperand::B => {
                                todo!();
                            },
                            RotateOperand::C => {
                                todo!();
                            },
                            RotateOperand::D => {
                                todo!();
                            },
                            RotateOperand::E => {
                                todo!();
                            },
                            RotateOperand::H => {
                                todo!();
                            },
                            RotateOperand::L => {
                                todo!();
                            },
                            RotateOperand::HLIndirect => {
                                todo!();
                            },
                        }
                    },
                }
            }
        };

        println!("{} cycles", cycles);
        cycles
    }

    // TODO implement
    pub fn pixel_buffer(&self) -> std::slice::Iter<'_, u8> {
        self.tmp_buffer.iter()
    }

    fn calculate_cycles(duration: u32) -> i32 {
        // XXX this might panic
        (duration/1000).try_into().unwrap()
    }

    pub fn run(&mut self, duration: u32) -> usize {
        let mut cycles_to_run = CPU::calculate_cycles(duration);
        let mut cycles_ran = 0;

        while self.counter > 0 {
            println!("emulating...");

            let cycles = self.execute();

            self.counter -= cycles as i32;
            cycles_to_run -= cycles as i32;
            println!("self.counter = {}", cycles);

            cycles_ran += cycles as usize;

            if cycles_to_run <= 0 {
                break;
            }

            if self.counter <= 0 {
                // TODO run interrupt tasks
                self.counter = 20;
                println!("running interrupts");
            }
        }

        cycles_ran
    }
}
