use std::fs::File;

use crate::registers::{Flag,Registers};
use crate::memory_bus::MemoryBus;

use crate::debug;

pub struct CPU {
    pub reg: Registers,
    pub memory_bus: MemoryBus,
    pub counter: i32,
    pub tmp_buffer: Vec<u8>,
    pub breakpoints: Vec<u16>,
    // is this all we need for HALT?
    running: bool,
    IME: bool,
    debug: bool,
    stepping: bool,
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
    RLC(PrefixOperand),
    RRC(PrefixOperand),
    RL(PrefixOperand),
    RR(PrefixOperand),
    SLA(PrefixOperand),
    SRA(PrefixOperand),
    SRL(PrefixOperand),
    SWAP(PrefixOperand),
    BIT(PrefixOperand, BitPosition),
    RES(PrefixOperand, BitPosition),
    SET(PrefixOperand, BitPosition),
}

#[derive(Debug)]
pub enum PrefixOperand {
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
pub enum BitPosition {
    B0,
    B1,
    B2,
    B3,
    B4,
    B5,
    B6,
    B7
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
            0x00 => Ok(PrefixedOpcode::RLC(PrefixOperand::B)),
            0x01 => Ok(PrefixedOpcode::RLC(PrefixOperand::C)),
            0x02 => Ok(PrefixedOpcode::RLC(PrefixOperand::D)),
            0x03 => Ok(PrefixedOpcode::RLC(PrefixOperand::E)),
            0x04 => Ok(PrefixedOpcode::RLC(PrefixOperand::H)),
            0x05 => Ok(PrefixedOpcode::RLC(PrefixOperand::L)),
            0x06 => Ok(PrefixedOpcode::RLC(PrefixOperand::HLIndirect)),
            0x07 => Ok(PrefixedOpcode::RLC(PrefixOperand::A)),
            0x08 => Ok(PrefixedOpcode::RRC(PrefixOperand::B)),
            0x09 => Ok(PrefixedOpcode::RRC(PrefixOperand::C)),
            0x0a => Ok(PrefixedOpcode::RRC(PrefixOperand::D)),
            0x0b => Ok(PrefixedOpcode::RRC(PrefixOperand::E)),
            0x0c => Ok(PrefixedOpcode::RRC(PrefixOperand::H)),
            0x0d => Ok(PrefixedOpcode::RRC(PrefixOperand::L)),
            0x0e => Ok(PrefixedOpcode::RRC(PrefixOperand::HLIndirect)),
            0x0f => Ok(PrefixedOpcode::RRC(PrefixOperand::A)),
            0x10 => Ok(PrefixedOpcode::RL(PrefixOperand::B)),
            0x11 => Ok(PrefixedOpcode::RL(PrefixOperand::C)),
            0x12 => Ok(PrefixedOpcode::RL(PrefixOperand::D)),
            0x13 => Ok(PrefixedOpcode::RL(PrefixOperand::E)),
            0x14 => Ok(PrefixedOpcode::RL(PrefixOperand::H)),
            0x15 => Ok(PrefixedOpcode::RL(PrefixOperand::L)),
            0x16 => Ok(PrefixedOpcode::RL(PrefixOperand::HLIndirect)),
            0x17 => Ok(PrefixedOpcode::RL(PrefixOperand::A)),
            0x18 => Ok(PrefixedOpcode::RR(PrefixOperand::B)),
            0x19 => Ok(PrefixedOpcode::RR(PrefixOperand::C)),
            0x1a => Ok(PrefixedOpcode::RR(PrefixOperand::D)),
            0x1b => Ok(PrefixedOpcode::RR(PrefixOperand::E)),
            0x1c => Ok(PrefixedOpcode::RR(PrefixOperand::H)),
            0x1d => Ok(PrefixedOpcode::RR(PrefixOperand::L)),
            0x1e => Ok(PrefixedOpcode::RR(PrefixOperand::HLIndirect)),
            0x1f => Ok(PrefixedOpcode::RR(PrefixOperand::A)),
            0x20 => Ok(PrefixedOpcode::SLA(PrefixOperand::B)),
            0x21 => Ok(PrefixedOpcode::SLA(PrefixOperand::C)),
            0x22 => Ok(PrefixedOpcode::SLA(PrefixOperand::D)),
            0x23 => Ok(PrefixedOpcode::SLA(PrefixOperand::E)),
            0x24 => Ok(PrefixedOpcode::SLA(PrefixOperand::H)),
            0x25 => Ok(PrefixedOpcode::SLA(PrefixOperand::L)),
            0x26 => Ok(PrefixedOpcode::SLA(PrefixOperand::HLIndirect)),
            0x27 => Ok(PrefixedOpcode::SLA(PrefixOperand::A)),
            0x28 => Ok(PrefixedOpcode::SRA(PrefixOperand::B)),
            0x29 => Ok(PrefixedOpcode::SRA(PrefixOperand::C)),
            0x2a => Ok(PrefixedOpcode::SRA(PrefixOperand::D)),
            0x2b => Ok(PrefixedOpcode::SRA(PrefixOperand::E)),
            0x2c => Ok(PrefixedOpcode::SRA(PrefixOperand::H)),
            0x2d => Ok(PrefixedOpcode::SRA(PrefixOperand::L)),
            0x2e => Ok(PrefixedOpcode::SRA(PrefixOperand::HLIndirect)),
            0x2f => Ok(PrefixedOpcode::SRA(PrefixOperand::A)),
            0x30 => Ok(PrefixedOpcode::SWAP(PrefixOperand::B)),
            0x31 => Ok(PrefixedOpcode::SWAP(PrefixOperand::C)),
            0x32 => Ok(PrefixedOpcode::SWAP(PrefixOperand::D)),
            0x33 => Ok(PrefixedOpcode::SWAP(PrefixOperand::E)),
            0x34 => Ok(PrefixedOpcode::SWAP(PrefixOperand::H)),
            0x35 => Ok(PrefixedOpcode::SWAP(PrefixOperand::L)),
            0x36 => Ok(PrefixedOpcode::SWAP(PrefixOperand::HLIndirect)),
            0x37 => Ok(PrefixedOpcode::SWAP(PrefixOperand::A)),
            0x38 => Ok(PrefixedOpcode::SRL(PrefixOperand::B)),
            0x39 => Ok(PrefixedOpcode::SRL(PrefixOperand::C)),
            0x3a => Ok(PrefixedOpcode::SRL(PrefixOperand::D)),
            0x3b => Ok(PrefixedOpcode::SRL(PrefixOperand::E)),
            0x3c => Ok(PrefixedOpcode::SRL(PrefixOperand::H)),
            0x3d => Ok(PrefixedOpcode::SRL(PrefixOperand::L)),
            0x3e => Ok(PrefixedOpcode::SRL(PrefixOperand::HLIndirect)),
            0x3f => Ok(PrefixedOpcode::SRL(PrefixOperand::A)),
            0x40 => Ok(PrefixedOpcode::BIT(PrefixOperand::B, BitPosition::B0)),
            0x41 => Ok(PrefixedOpcode::BIT(PrefixOperand::C, BitPosition::B0)),
            0x42 => Ok(PrefixedOpcode::BIT(PrefixOperand::D, BitPosition::B0)),
            0x43 => Ok(PrefixedOpcode::BIT(PrefixOperand::E, BitPosition::B0)),
            0x44 => Ok(PrefixedOpcode::BIT(PrefixOperand::H, BitPosition::B0)),
            0x45 => Ok(PrefixedOpcode::BIT(PrefixOperand::L, BitPosition::B0)),
            0x46 => Ok(PrefixedOpcode::BIT(PrefixOperand::HLIndirect, BitPosition::B0)),
            0x47 => Ok(PrefixedOpcode::BIT(PrefixOperand::A, BitPosition::B0)),
            0x48 => Ok(PrefixedOpcode::BIT(PrefixOperand::B, BitPosition::B1)),
            0x49 => Ok(PrefixedOpcode::BIT(PrefixOperand::C, BitPosition::B1)),
            0x4a => Ok(PrefixedOpcode::BIT(PrefixOperand::D, BitPosition::B1)),
            0x4b => Ok(PrefixedOpcode::BIT(PrefixOperand::E, BitPosition::B1)),
            0x4c => Ok(PrefixedOpcode::BIT(PrefixOperand::H, BitPosition::B1)),
            0x4d => Ok(PrefixedOpcode::BIT(PrefixOperand::L, BitPosition::B1)),
            0x4e => Ok(PrefixedOpcode::BIT(PrefixOperand::HLIndirect, BitPosition::B1)),
            0x4f => Ok(PrefixedOpcode::BIT(PrefixOperand::A, BitPosition::B1)),
            0x50 => Ok(PrefixedOpcode::BIT(PrefixOperand::B, BitPosition::B2)),
            0x51 => Ok(PrefixedOpcode::BIT(PrefixOperand::C, BitPosition::B2)),
            0x52 => Ok(PrefixedOpcode::BIT(PrefixOperand::D, BitPosition::B2)),
            0x53 => Ok(PrefixedOpcode::BIT(PrefixOperand::E, BitPosition::B2)),
            0x54 => Ok(PrefixedOpcode::BIT(PrefixOperand::H, BitPosition::B2)),
            0x55 => Ok(PrefixedOpcode::BIT(PrefixOperand::L, BitPosition::B2)),
            0x56 => Ok(PrefixedOpcode::BIT(PrefixOperand::HLIndirect, BitPosition::B2)),
            0x57 => Ok(PrefixedOpcode::BIT(PrefixOperand::A, BitPosition::B2)),
            0x58 => Ok(PrefixedOpcode::BIT(PrefixOperand::B, BitPosition::B3)),
            0x59 => Ok(PrefixedOpcode::BIT(PrefixOperand::C, BitPosition::B3)),
            0x5a => Ok(PrefixedOpcode::BIT(PrefixOperand::D, BitPosition::B3)),
            0x5b => Ok(PrefixedOpcode::BIT(PrefixOperand::E, BitPosition::B3)),
            0x5c => Ok(PrefixedOpcode::BIT(PrefixOperand::H, BitPosition::B3)),
            0x5d => Ok(PrefixedOpcode::BIT(PrefixOperand::L, BitPosition::B3)),
            0x5e => Ok(PrefixedOpcode::BIT(PrefixOperand::HLIndirect, BitPosition::B3)),
            0x5f => Ok(PrefixedOpcode::BIT(PrefixOperand::A, BitPosition::B3)),
            0x60 => Ok(PrefixedOpcode::BIT(PrefixOperand::B, BitPosition::B4)),
            0x61 => Ok(PrefixedOpcode::BIT(PrefixOperand::C, BitPosition::B4)),
            0x62 => Ok(PrefixedOpcode::BIT(PrefixOperand::D, BitPosition::B4)),
            0x63 => Ok(PrefixedOpcode::BIT(PrefixOperand::E, BitPosition::B4)),
            0x64 => Ok(PrefixedOpcode::BIT(PrefixOperand::H, BitPosition::B4)),
            0x65 => Ok(PrefixedOpcode::BIT(PrefixOperand::L, BitPosition::B4)),
            0x66 => Ok(PrefixedOpcode::BIT(PrefixOperand::HLIndirect, BitPosition::B4)),
            0x67 => Ok(PrefixedOpcode::BIT(PrefixOperand::A, BitPosition::B4)),
            0x68 => Ok(PrefixedOpcode::BIT(PrefixOperand::B, BitPosition::B5)),
            0x69 => Ok(PrefixedOpcode::BIT(PrefixOperand::C, BitPosition::B5)),
            0x6a => Ok(PrefixedOpcode::BIT(PrefixOperand::D, BitPosition::B5)),
            0x6b => Ok(PrefixedOpcode::BIT(PrefixOperand::E, BitPosition::B5)),
            0x6c => Ok(PrefixedOpcode::BIT(PrefixOperand::H, BitPosition::B5)),
            0x6d => Ok(PrefixedOpcode::BIT(PrefixOperand::L, BitPosition::B5)),
            0x6e => Ok(PrefixedOpcode::BIT(PrefixOperand::HLIndirect, BitPosition::B5)),
            0x6f => Ok(PrefixedOpcode::BIT(PrefixOperand::A, BitPosition::B5)),
            0x70 => Ok(PrefixedOpcode::BIT(PrefixOperand::B, BitPosition::B6)),
            0x71 => Ok(PrefixedOpcode::BIT(PrefixOperand::C, BitPosition::B6)),
            0x72 => Ok(PrefixedOpcode::BIT(PrefixOperand::D, BitPosition::B6)),
            0x73 => Ok(PrefixedOpcode::BIT(PrefixOperand::E, BitPosition::B6)),
            0x74 => Ok(PrefixedOpcode::BIT(PrefixOperand::H, BitPosition::B6)),
            0x75 => Ok(PrefixedOpcode::BIT(PrefixOperand::L, BitPosition::B6)),
            0x76 => Ok(PrefixedOpcode::BIT(PrefixOperand::HLIndirect, BitPosition::B6)),
            0x77 => Ok(PrefixedOpcode::BIT(PrefixOperand::A, BitPosition::B6)),
            0x78 => Ok(PrefixedOpcode::BIT(PrefixOperand::B, BitPosition::B7)),
            0x79 => Ok(PrefixedOpcode::BIT(PrefixOperand::C, BitPosition::B7)),
            0x7a => Ok(PrefixedOpcode::BIT(PrefixOperand::D, BitPosition::B7)),
            0x7b => Ok(PrefixedOpcode::BIT(PrefixOperand::E, BitPosition::B7)),
            0x7c => Ok(PrefixedOpcode::BIT(PrefixOperand::H, BitPosition::B7)),
            0x7d => Ok(PrefixedOpcode::BIT(PrefixOperand::L, BitPosition::B7)),
            0x7e => Ok(PrefixedOpcode::BIT(PrefixOperand::HLIndirect, BitPosition::B7)),
            0x7f => Ok(PrefixedOpcode::BIT(PrefixOperand::A, BitPosition::B7)),
            0x80 => Ok(PrefixedOpcode::RES(PrefixOperand::B, BitPosition::B0)),
            0x81 => Ok(PrefixedOpcode::RES(PrefixOperand::C, BitPosition::B0)),
            0x82 => Ok(PrefixedOpcode::RES(PrefixOperand::D, BitPosition::B0)),
            0x83 => Ok(PrefixedOpcode::RES(PrefixOperand::E, BitPosition::B0)),
            0x84 => Ok(PrefixedOpcode::RES(PrefixOperand::H, BitPosition::B0)),
            0x85 => Ok(PrefixedOpcode::RES(PrefixOperand::L, BitPosition::B0)),
            0x86 => Ok(PrefixedOpcode::RES(PrefixOperand::HLIndirect, BitPosition::B0)),
            0x87 => Ok(PrefixedOpcode::RES(PrefixOperand::A, BitPosition::B0)),
            0x88 => Ok(PrefixedOpcode::RES(PrefixOperand::B, BitPosition::B1)),
            0x89 => Ok(PrefixedOpcode::RES(PrefixOperand::C, BitPosition::B1)),
            0x8a => Ok(PrefixedOpcode::RES(PrefixOperand::D, BitPosition::B1)),
            0x8b => Ok(PrefixedOpcode::RES(PrefixOperand::E, BitPosition::B1)),
            0x8c => Ok(PrefixedOpcode::RES(PrefixOperand::H, BitPosition::B1)),
            0x8d => Ok(PrefixedOpcode::RES(PrefixOperand::L, BitPosition::B1)),
            0x8e => Ok(PrefixedOpcode::RES(PrefixOperand::HLIndirect, BitPosition::B1)),
            0x8f => Ok(PrefixedOpcode::RES(PrefixOperand::A, BitPosition::B1)),
            0x90 => Ok(PrefixedOpcode::RES(PrefixOperand::B, BitPosition::B2)),
            0x91 => Ok(PrefixedOpcode::RES(PrefixOperand::C, BitPosition::B2)),
            0x92 => Ok(PrefixedOpcode::RES(PrefixOperand::D, BitPosition::B2)),
            0x93 => Ok(PrefixedOpcode::RES(PrefixOperand::E, BitPosition::B2)),
            0x94 => Ok(PrefixedOpcode::RES(PrefixOperand::H, BitPosition::B2)),
            0x95 => Ok(PrefixedOpcode::RES(PrefixOperand::L, BitPosition::B2)),
            0x96 => Ok(PrefixedOpcode::RES(PrefixOperand::HLIndirect, BitPosition::B2)),
            0x97 => Ok(PrefixedOpcode::RES(PrefixOperand::A, BitPosition::B2)),
            0x98 => Ok(PrefixedOpcode::RES(PrefixOperand::B, BitPosition::B3)),
            0x99 => Ok(PrefixedOpcode::RES(PrefixOperand::C, BitPosition::B3)),
            0x9a => Ok(PrefixedOpcode::RES(PrefixOperand::D, BitPosition::B3)),
            0x9b => Ok(PrefixedOpcode::RES(PrefixOperand::E, BitPosition::B3)),
            0x9c => Ok(PrefixedOpcode::RES(PrefixOperand::H, BitPosition::B3)),
            0x9d => Ok(PrefixedOpcode::RES(PrefixOperand::L, BitPosition::B3)),
            0x9e => Ok(PrefixedOpcode::RES(PrefixOperand::HLIndirect, BitPosition::B3)),
            0x9f => Ok(PrefixedOpcode::RES(PrefixOperand::A, BitPosition::B3)),
            0xa0 => Ok(PrefixedOpcode::RES(PrefixOperand::B, BitPosition::B4)),
            0xa1 => Ok(PrefixedOpcode::RES(PrefixOperand::C, BitPosition::B4)),
            0xa2 => Ok(PrefixedOpcode::RES(PrefixOperand::D, BitPosition::B4)),
            0xa3 => Ok(PrefixedOpcode::RES(PrefixOperand::E, BitPosition::B4)),
            0xa4 => Ok(PrefixedOpcode::RES(PrefixOperand::H, BitPosition::B4)),
            0xa5 => Ok(PrefixedOpcode::RES(PrefixOperand::L, BitPosition::B4)),
            0xa6 => Ok(PrefixedOpcode::RES(PrefixOperand::HLIndirect, BitPosition::B4)),
            0xa7 => Ok(PrefixedOpcode::RES(PrefixOperand::A, BitPosition::B4)),
            0xa8 => Ok(PrefixedOpcode::RES(PrefixOperand::B, BitPosition::B5)),
            0xa9 => Ok(PrefixedOpcode::RES(PrefixOperand::C, BitPosition::B5)),
            0xaa => Ok(PrefixedOpcode::RES(PrefixOperand::D, BitPosition::B5)),
            0xab => Ok(PrefixedOpcode::RES(PrefixOperand::E, BitPosition::B5)),
            0xac => Ok(PrefixedOpcode::RES(PrefixOperand::H, BitPosition::B5)),
            0xad => Ok(PrefixedOpcode::RES(PrefixOperand::L, BitPosition::B5)),
            0xae => Ok(PrefixedOpcode::RES(PrefixOperand::HLIndirect, BitPosition::B5)),
            0xaf => Ok(PrefixedOpcode::RES(PrefixOperand::A, BitPosition::B5)),
            0xb0 => Ok(PrefixedOpcode::RES(PrefixOperand::B, BitPosition::B6)),
            0xb1 => Ok(PrefixedOpcode::RES(PrefixOperand::C, BitPosition::B6)),
            0xb2 => Ok(PrefixedOpcode::RES(PrefixOperand::D, BitPosition::B6)),
            0xb3 => Ok(PrefixedOpcode::RES(PrefixOperand::E, BitPosition::B6)),
            0xb4 => Ok(PrefixedOpcode::RES(PrefixOperand::H, BitPosition::B6)),
            0xb5 => Ok(PrefixedOpcode::RES(PrefixOperand::L, BitPosition::B6)),
            0xb6 => Ok(PrefixedOpcode::RES(PrefixOperand::HLIndirect, BitPosition::B6)),
            0xb7 => Ok(PrefixedOpcode::RES(PrefixOperand::A, BitPosition::B6)),
            0xb8 => Ok(PrefixedOpcode::RES(PrefixOperand::B, BitPosition::B7)),
            0xb9 => Ok(PrefixedOpcode::RES(PrefixOperand::C, BitPosition::B7)),
            0xba => Ok(PrefixedOpcode::RES(PrefixOperand::D, BitPosition::B7)),
            0xbb => Ok(PrefixedOpcode::RES(PrefixOperand::E, BitPosition::B7)),
            0xbc => Ok(PrefixedOpcode::RES(PrefixOperand::H, BitPosition::B7)),
            0xbd => Ok(PrefixedOpcode::RES(PrefixOperand::L, BitPosition::B7)),
            0xbe => Ok(PrefixedOpcode::RES(PrefixOperand::HLIndirect, BitPosition::B7)),
            0xbf => Ok(PrefixedOpcode::RES(PrefixOperand::A, BitPosition::B7)),
            0xc0 => Ok(PrefixedOpcode::SET(PrefixOperand::B, BitPosition::B0)),
            0xc1 => Ok(PrefixedOpcode::SET(PrefixOperand::C, BitPosition::B0)),
            0xc2 => Ok(PrefixedOpcode::SET(PrefixOperand::D, BitPosition::B0)),
            0xc3 => Ok(PrefixedOpcode::SET(PrefixOperand::E, BitPosition::B0)),
            0xc4 => Ok(PrefixedOpcode::SET(PrefixOperand::H, BitPosition::B0)),
            0xc5 => Ok(PrefixedOpcode::SET(PrefixOperand::L, BitPosition::B0)),
            0xc6 => Ok(PrefixedOpcode::SET(PrefixOperand::HLIndirect, BitPosition::B0)),
            0xc7 => Ok(PrefixedOpcode::SET(PrefixOperand::A, BitPosition::B0)),
            0xc8 => Ok(PrefixedOpcode::SET(PrefixOperand::B, BitPosition::B1)),
            0xc9 => Ok(PrefixedOpcode::SET(PrefixOperand::C, BitPosition::B1)),
            0xca => Ok(PrefixedOpcode::SET(PrefixOperand::D, BitPosition::B1)),
            0xcb => Ok(PrefixedOpcode::SET(PrefixOperand::E, BitPosition::B1)),
            0xcc => Ok(PrefixedOpcode::SET(PrefixOperand::H, BitPosition::B1)),
            0xcd => Ok(PrefixedOpcode::SET(PrefixOperand::L, BitPosition::B1)),
            0xce => Ok(PrefixedOpcode::SET(PrefixOperand::HLIndirect, BitPosition::B1)),
            0xcf => Ok(PrefixedOpcode::SET(PrefixOperand::A, BitPosition::B1)),
            0xd0 => Ok(PrefixedOpcode::SET(PrefixOperand::B, BitPosition::B2)),
            0xd1 => Ok(PrefixedOpcode::SET(PrefixOperand::C, BitPosition::B2)),
            0xd2 => Ok(PrefixedOpcode::SET(PrefixOperand::D, BitPosition::B2)),
            0xd3 => Ok(PrefixedOpcode::SET(PrefixOperand::E, BitPosition::B2)),
            0xd4 => Ok(PrefixedOpcode::SET(PrefixOperand::H, BitPosition::B2)),
            0xd5 => Ok(PrefixedOpcode::SET(PrefixOperand::L, BitPosition::B2)),
            0xd6 => Ok(PrefixedOpcode::SET(PrefixOperand::HLIndirect, BitPosition::B2)),
            0xd7 => Ok(PrefixedOpcode::SET(PrefixOperand::A, BitPosition::B2)),
            0xd8 => Ok(PrefixedOpcode::SET(PrefixOperand::B, BitPosition::B3)),
            0xd9 => Ok(PrefixedOpcode::SET(PrefixOperand::C, BitPosition::B3)),
            0xda => Ok(PrefixedOpcode::SET(PrefixOperand::D, BitPosition::B3)),
            0xdb => Ok(PrefixedOpcode::SET(PrefixOperand::E, BitPosition::B3)),
            0xdc => Ok(PrefixedOpcode::SET(PrefixOperand::H, BitPosition::B3)),
            0xdd => Ok(PrefixedOpcode::SET(PrefixOperand::L, BitPosition::B3)),
            0xde => Ok(PrefixedOpcode::SET(PrefixOperand::HLIndirect, BitPosition::B3)),
            0xdf => Ok(PrefixedOpcode::SET(PrefixOperand::A, BitPosition::B3)),
            0xe0 => Ok(PrefixedOpcode::SET(PrefixOperand::B, BitPosition::B4)),
            0xe1 => Ok(PrefixedOpcode::SET(PrefixOperand::C, BitPosition::B4)),
            0xe2 => Ok(PrefixedOpcode::SET(PrefixOperand::D, BitPosition::B4)),
            0xe3 => Ok(PrefixedOpcode::SET(PrefixOperand::E, BitPosition::B4)),
            0xe4 => Ok(PrefixedOpcode::SET(PrefixOperand::H, BitPosition::B4)),
            0xe5 => Ok(PrefixedOpcode::SET(PrefixOperand::L, BitPosition::B4)),
            0xe6 => Ok(PrefixedOpcode::SET(PrefixOperand::HLIndirect, BitPosition::B4)),
            0xe7 => Ok(PrefixedOpcode::SET(PrefixOperand::A, BitPosition::B4)),
            0xe8 => Ok(PrefixedOpcode::SET(PrefixOperand::B, BitPosition::B5)),
            0xe9 => Ok(PrefixedOpcode::SET(PrefixOperand::C, BitPosition::B5)),
            0xea => Ok(PrefixedOpcode::SET(PrefixOperand::D, BitPosition::B5)),
            0xeb => Ok(PrefixedOpcode::SET(PrefixOperand::E, BitPosition::B5)),
            0xec => Ok(PrefixedOpcode::SET(PrefixOperand::H, BitPosition::B5)),
            0xed => Ok(PrefixedOpcode::SET(PrefixOperand::L, BitPosition::B5)),
            0xee => Ok(PrefixedOpcode::SET(PrefixOperand::HLIndirect, BitPosition::B5)),
            0xef => Ok(PrefixedOpcode::SET(PrefixOperand::A, BitPosition::B5)),
            0xf0 => Ok(PrefixedOpcode::SET(PrefixOperand::B, BitPosition::B6)),
            0xf1 => Ok(PrefixedOpcode::SET(PrefixOperand::C, BitPosition::B6)),
            0xf2 => Ok(PrefixedOpcode::SET(PrefixOperand::D, BitPosition::B6)),
            0xf3 => Ok(PrefixedOpcode::SET(PrefixOperand::E, BitPosition::B6)),
            0xf4 => Ok(PrefixedOpcode::SET(PrefixOperand::H, BitPosition::B6)),
            0xf5 => Ok(PrefixedOpcode::SET(PrefixOperand::L, BitPosition::B6)),
            0xf6 => Ok(PrefixedOpcode::SET(PrefixOperand::HLIndirect, BitPosition::B6)),
            0xf7 => Ok(PrefixedOpcode::SET(PrefixOperand::A, BitPosition::B6)),
            0xf8 => Ok(PrefixedOpcode::SET(PrefixOperand::B, BitPosition::B7)),
            0xf9 => Ok(PrefixedOpcode::SET(PrefixOperand::C, BitPosition::B7)),
            0xfa => Ok(PrefixedOpcode::SET(PrefixOperand::D, BitPosition::B7)),
            0xfb => Ok(PrefixedOpcode::SET(PrefixOperand::E, BitPosition::B7)),
            0xfc => Ok(PrefixedOpcode::SET(PrefixOperand::H, BitPosition::B7)),
            0xfd => Ok(PrefixedOpcode::SET(PrefixOperand::L, BitPosition::B7)),
            0xfe => Ok(PrefixedOpcode::SET(PrefixOperand::HLIndirect, BitPosition::B7)),
            0xff => Ok(PrefixedOpcode::SET(PrefixOperand::A, BitPosition::B7)),
        }
    }
}

impl CPU {
    pub fn new(rom_path: String, debug: bool) -> CPU {
        let mut cpu = CPU {
            reg: Registers::new(),
            counter: 20,
            memory_bus: MemoryBus::new(),
            // TODO remove
            tmp_buffer: vec![1; 100],
            breakpoints: vec![],
            IME: true,
            running: true,
            debug: debug,
            stepping: false,
        };

        cpu.breakpoints.push(0x0);

        // TODO error handling

        // FIXME pass this from main
        let boot_rom_path = "/home/iaguis/programming/gbemu-rs/DMG_ROM.bin";
        let boot_rom_f = File::open(boot_rom_path).expect("can't open boot ROM");
        cpu.memory_bus.read_boot_rom(boot_rom_f).expect("can't read ROM");
        let rom_f = File::open(rom_path).expect("can't open ROM");
        cpu.memory_bus.read_rom(rom_f).expect("can't read ROM");

        cpu
    }

    #[inline(always)]
    fn log_debug(&self, message: String) {
        if self.debug {
            eprintln!("{message}");
        }
    }

    fn fetch_byte(&mut self) -> Result<Opcode, &'static str> {
        let b = self.memory_bus.read_byte(self.reg.pc.into());
        self.log_debug(format!("pc = {:#04x}", self.reg.pc));
        self.log_debug(format!("mem[pc] = {:#04x}", b));

        let opcode = Opcode::try_from(b)?;
        Ok(opcode)
    }

    fn fetch_prefixed_byte(&mut self) -> Result<PrefixedOpcode, &'static str> {
        let b = self.memory_bus.read_byte(self.reg.pc.into());
        self.log_debug(format!("pc = {:#04x}", self.reg.pc));
        self.log_debug(format!("mem[pc] = {:#04x}", b));

        let prefixed_opcode = PrefixedOpcode::try_from(b)?;
        Ok(prefixed_opcode)
    }

    // TODO double-check cycles
    fn execute(&mut self) -> u8 {
        // XXX this panics if it fails to decode the opcode, which is probably fine
        let opcode = self.fetch_byte().expect("failed fetching");

        if self.debug && (self.breakpoints.contains(&self.reg.pc)
        | self.stepping) {
            let r = debug::drop_to_shell(self);
            match r {
                Ok(ret) => match ret {
                    debug::DebuggerRet::Step => self.stepping = true,
                    _ => self.stepping = false,
                }
                Err(_) => panic!("error dropping to shell!"),
            }
        }

        let mut cycles = 1;

        self.log_debug(format!("opcode: {:?}", opcode));

        match opcode {
            Opcode::NOP => {
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

                        self.memory_bus.write_byte(address, self.reg.a);

                        cycles = 4;
                        self.reg.pc += 3;
                    },

                    LDType::AFromAddress => {
                        let msb = self.memory_bus.read_byte(self.reg.pc + 2);
                        let lsb = self.memory_bus.read_byte(self.reg.pc + 1);
                        let address = ((msb as u16) << 8) | lsb as u16;

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
                        let r = self.reg.alu_inc(val);
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
                match operand {
                    ALUOperand::D8 => { cycles = 2; self.reg.pc += 2; },
                    ALUOperand::HLIndirect => { cycles = 2; self.reg.pc += 1; },
                    _ => { cycles = 1; self.reg.pc += 1; }
                }
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
                cycles = 2;
                self.reg.pc += 1;
            },

            Opcode::ADDSP => {
                let val = self.memory_bus.read_byte(self.reg.pc + 1);
                self.reg.alu_addsp(val);
                cycles = 4;
                self.reg.pc += 2;
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
                match operand {
                    ALUOperand::D8 => { cycles = 2; self.reg.pc += 2; },
                    ALUOperand::HLIndirect => { cycles = 2; self.reg.pc += 1; },
                    _ => { cycles = 1; self.reg.pc += 1; }
                }
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
                match operand {
                    ALUOperand::D8 => { cycles = 2; self.reg.pc += 2; },
                    ALUOperand::HLIndirect => { cycles = 2; self.reg.pc += 1; },
                    _ => { cycles = 1; self.reg.pc += 1; }
                }
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
                match operand {
                    ALUOperand::D8 => { cycles = 2; self.reg.pc += 2; },
                    ALUOperand::HLIndirect => { cycles = 2; self.reg.pc += 1; },
                    _ => { cycles = 1; self.reg.pc += 1; }
                }
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
                match operand {
                    ALUOperand::D8 => { cycles = 2; self.reg.pc += 2; },
                    ALUOperand::HLIndirect => { cycles = 2; self.reg.pc += 1; },
                    _ => { cycles = 1; self.reg.pc += 1; }
                }
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
                match operand {
                    ALUOperand::D8 => { cycles = 2; self.reg.pc += 2; },
                    ALUOperand::HLIndirect => { cycles = 2; self.reg.pc += 1; },
                    _ => { cycles = 1; self.reg.pc += 1; }
                }
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
                match operand {
                    ALUOperand::D8 => { cycles = 2; self.reg.pc += 2; },
                    ALUOperand::HLIndirect => { cycles = 2; self.reg.pc += 1; },
                    _ => { cycles = 1; self.reg.pc += 1; }
                }
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
                match operand {
                    ALUOperand::D8 => { cycles = 2; self.reg.pc += 2; },
                    ALUOperand::HLIndirect => { cycles = 2; self.reg.pc += 1; },
                    _ => { cycles = 1; self.reg.pc += 1; }
                }
            }

            Opcode::RLCA => {
                let c = (self.reg.a & 0x80) >> 7;
                let r = self.reg.a.rotate_left(1) | c;

                self.reg.set_flag(Flag::Z, false);
                self.reg.set_flag(Flag::N, false);
                self.reg.set_flag(Flag::H, false);
                self.reg.set_flag(Flag::C, c == 0x01);

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
                let r = self.reg.a.rotate_right(1);

                self.reg.set_flag(Flag::Z, false);
                self.reg.set_flag(Flag::N, false);
                self.reg.set_flag(Flag::H, false);
                self.reg.set_flag(Flag::C, self.reg.a & 0x01 == 0x01);

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

            Opcode::DAA => {
                let mut a = self.reg.a;
                let c = self.reg.get_flag(Flag::C);
                let h = self.reg.get_flag(Flag::H);
                let n = self.reg.get_flag(Flag::N);

                if !n {
                    if c || a > 0x99 {
                        a = a.wrapping_add(0x60);
                        self.reg.set_flag(Flag::C, true);
                    }
                    if h || (a & 0x0f) > 0x09 {
                        a = a.wrapping_add(0x6);
                    }
                } else {
                    if c {
                        a -= 0x60;
                    }
                    if h {
                        a -= 0x6;
                    }
                }

                self.reg.set_flag(Flag::Z, a == 0);
                self.reg.set_flag(Flag::H, false);

                cycles = 1;
                self.reg.a = a;
                self.reg.pc += 1;
            },

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
                let next_instruction = self.reg.pc + 2;

                let jp_address = if offset >= 0 {
                    next_instruction.wrapping_add(offset as u16)
                } else {
                    next_instruction.wrapping_sub(offset.abs() as u16)
                };

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
                            self.reg.pc += 2;
                            cycles = 3;
                        }
                    },
                    JCondition::NC => {
                        if !self.reg.get_flag(Flag::C) {
                            self.reg.pc = jp_address;
                            cycles = 4;
                        } else {
                            self.reg.pc += 2;
                            cycles = 3;
                        }
                    },
                    JCondition::Z => {
                        if self.reg.get_flag(Flag::Z) {
                            self.reg.pc = jp_address;
                            cycles = 4;
                        } else {
                            self.reg.pc += 2;
                            cycles = 3;
                        }
                    },
                    JCondition::C => {
                        if self.reg.get_flag(Flag::C) {
                            self.reg.pc = jp_address;
                            cycles = 4;
                        } else {
                            self.reg.pc += 2;
                            cycles = 3;
                        }
                    },
                }
            },

            // TODO refactor
            Opcode::CALL(condition) => {
                let next_instruction = self.reg.pc + 3;
                match condition  {
                    JCondition::Nothing => {
                        let msb = self.memory_bus.read_byte(self.reg.pc + 2);
                        let lsb = self.memory_bus.read_byte(self.reg.pc + 1);

                        self.reg.sp -= 1;
                        self.memory_bus.write_byte(self.reg.sp, (next_instruction >> 8) as u8);
                        self.reg.sp -= 1;
                        self.memory_bus.write_byte(self.reg.sp, (next_instruction & 0xFF) as u8);

                        self.reg.pc = ((msb as u16) << 8) | (lsb as u16);
                        cycles = 6;
                    },
                    JCondition::NZ => {
                        let msb = self.memory_bus.read_byte(self.reg.pc + 2);
                        let lsb = self.memory_bus.read_byte(self.reg.pc + 1);

                        if !self.reg.get_flag(Flag::Z) {
                            self.reg.sp -= 1;
                            self.memory_bus.write_byte(self.reg.sp, (next_instruction >> 8) as u8);
                            self.reg.sp -= 1;
                            self.memory_bus.write_byte(self.reg.sp, (next_instruction & 0xFF) as u8);

                            self.reg.pc = ((msb as u16) << 8) | (lsb as u16);
                            cycles = 6;
                        } else {
                            cycles = 3;
                            self.reg.pc += 3;
                        }
                    },
                    JCondition::NC => {
                        let msb = self.memory_bus.read_byte(self.reg.pc + 2);
                        let lsb = self.memory_bus.read_byte(self.reg.pc + 1);

                        if !self.reg.get_flag(Flag::C) {
                            self.reg.sp -= 1;
                            self.memory_bus.write_byte(self.reg.sp, (next_instruction >> 8) as u8);
                            self.reg.sp -= 1;
                            self.memory_bus.write_byte(self.reg.sp, (next_instruction & 0xFF) as u8);

                            self.reg.pc = ((msb as u16) << 8) | (lsb as u16);
                            cycles = 6;
                        } else {
                            cycles = 3;
                            self.reg.pc += 3;
                        }
                    },
                    JCondition::Z => {
                        let msb = self.memory_bus.read_byte(self.reg.pc + 2);
                        let lsb = self.memory_bus.read_byte(self.reg.pc + 1);

                        if self.reg.get_flag(Flag::Z) {
                            self.reg.sp -= 1;
                            self.memory_bus.write_byte(self.reg.sp, (next_instruction >> 8) as u8);
                            self.reg.sp -= 1;
                            self.memory_bus.write_byte(self.reg.sp, (next_instruction & 0xFF) as u8);

                            self.reg.pc = ((msb as u16) << 8) | (lsb as u16);
                            cycles = 6;
                        } else {
                            cycles = 3;
                            self.reg.pc += 3;
                        }
                    },
                    JCondition::C => {
                        let msb = self.memory_bus.read_byte(self.reg.pc + 2);
                        let lsb = self.memory_bus.read_byte(self.reg.pc + 1);

                        if self.reg.get_flag(Flag::C) {
                            self.reg.sp -= 1;
                            self.memory_bus.write_byte(self.reg.sp, (next_instruction >> 8) as u8);
                            self.reg.sp -= 1;
                            self.memory_bus.write_byte(self.reg.sp, (next_instruction & 0xFF) as u8);

                            self.reg.pc = ((msb as u16) << 8) | (lsb as u16);
                            cycles = 6;
                        } else {
                            cycles = 3;
                            self.reg.pc += 3;
                        }
                    },
                }
            }

            // TODO refactor
            Opcode::RET(condition) => {
                match condition {
                    JCondition::Nothing => {
                        let lsb = self.memory_bus.read_byte(self.reg.sp);
                        self.reg.sp += 1;
                        let msb = self.memory_bus.read_byte(self.reg.sp);
                        self.reg.sp += 1;

                        cycles = 4;
                        self.reg.pc = ((msb as u16) << 8) | (lsb as u16);
                    },
                    JCondition::NZ => {
                        if !self.reg.get_flag(Flag::Z) {
                            let lsb = self.memory_bus.read_byte(self.reg.sp);
                            self.reg.sp += 1;
                            let msb = self.memory_bus.read_byte(self.reg.sp);
                            self.reg.sp += 1;

                            cycles = 5;
                            self.reg.pc = ((msb as u16) << 8) | (lsb as u16);
                        } else {
                            cycles = 2;
                            self.reg.pc += 1;
                        }
                    }
                    JCondition::NC => {
                        if !self.reg.get_flag(Flag::C) {
                            let lsb = self.memory_bus.read_byte(self.reg.sp);
                            self.reg.sp += 1;
                            let msb = self.memory_bus.read_byte(self.reg.sp);
                            self.reg.sp += 1;

                            cycles = 5;
                            self.reg.pc = ((msb as u16) << 8) | (lsb as u16);
                        } else {
                            cycles = 2;
                            self.reg.pc += 1;
                        }
                    }
                    JCondition::Z => {
                        if self.reg.get_flag(Flag::Z) {
                            let lsb = self.memory_bus.read_byte(self.reg.sp);
                            self.reg.sp += 1;
                            let msb = self.memory_bus.read_byte(self.reg.sp);
                            self.reg.sp += 1;

                            cycles = 5;
                            self.reg.pc = ((msb as u16) << 8) | (lsb as u16);
                        } else {
                            cycles = 2;
                            self.reg.pc += 1;
                        }
                    }
                    JCondition::C => {
                        if self.reg.get_flag(Flag::C) {
                            let lsb = self.memory_bus.read_byte(self.reg.sp);
                            self.reg.sp += 1;
                            let msb = self.memory_bus.read_byte(self.reg.sp);
                            self.reg.sp += 1;

                            cycles = 5;
                            self.reg.pc = ((msb as u16) << 8) | (lsb as u16);
                        } else {
                            cycles = 2;
                            self.reg.pc += 1;
                        }
                    }
                }
            }

            Opcode::RETI => {
                let lsb = self.memory_bus.read_byte(self.reg.sp);
                self.reg.sp += 1;
                let msb = self.memory_bus.read_byte(self.reg.sp);
                self.reg.sp += 1;

                cycles = 4;
                self.reg.pc = ((msb as u16) << 8) | (lsb as u16);
                self.IME = true;
            },

            Opcode::RST(address) => {
                let next_instruction = self.reg.pc + 1;
                match address {
                    RSTAddress::X00 => {
                        let n = 0x0000;
                        self.reg.sp -= 1;
                        self.memory_bus.write_byte(self.reg.sp, (next_instruction >> 8) as u8);
                        self.reg.sp -= 1;
                        self.memory_bus.write_byte(self.reg.sp, (next_instruction & 0xFF) as u8);

                        cycles = 4;
                        self.reg.pc = n;
                    }
                    RSTAddress::X10 => {
                        let n = 0x0010;
                        self.reg.sp -= 1;
                        self.memory_bus.write_byte(self.reg.sp, (next_instruction >> 8) as u8);
                        self.reg.sp -= 1;
                        self.memory_bus.write_byte(self.reg.sp, (next_instruction & 0xFF) as u8);

                        cycles = 4;
                        self.reg.pc = n;
                    }
                    RSTAddress::X20 => {
                        let n = 0x0020;
                        self.reg.sp -= 1;
                        self.memory_bus.write_byte(self.reg.sp, (next_instruction >> 8) as u8);
                        self.reg.sp -= 1;
                        self.memory_bus.write_byte(self.reg.sp, (next_instruction & 0xFF) as u8);

                        cycles = 4;
                        self.reg.pc = n;
                    }
                    RSTAddress::X30 => {
                        let n = 0x0030;
                        self.reg.sp -= 1;
                        self.memory_bus.write_byte(self.reg.sp, (next_instruction >> 8) as u8);
                        self.reg.sp -= 1;
                        self.memory_bus.write_byte(self.reg.sp, (next_instruction & 0xFF) as u8);

                        cycles = 4;
                        self.reg.pc = n;
                    }
                    RSTAddress::X08 => {
                        let n = 0x0008;
                        self.reg.sp -= 1;
                        self.memory_bus.write_byte(self.reg.sp, (next_instruction >> 8) as u8);
                        self.reg.sp -= 1;
                        self.memory_bus.write_byte(self.reg.sp, (next_instruction & 0xFF) as u8);

                        cycles = 4;
                        self.reg.pc = n;
                    }
                    RSTAddress::X18 => {
                        let n = 0x0018;
                        self.reg.sp -= 1;
                        self.memory_bus.write_byte(self.reg.sp, (next_instruction >> 8) as u8);
                        self.reg.sp -= 1;
                        self.memory_bus.write_byte(self.reg.sp, (next_instruction & 0xFF) as u8);

                        cycles = 4;
                        self.reg.pc = n;
                    }
                    RSTAddress::X28 => {
                        let n = 0x0028;
                        self.reg.sp -= 1;
                        self.memory_bus.write_byte(self.reg.sp, (next_instruction >> 8) as u8);
                        self.reg.sp -= 1;
                        self.memory_bus.write_byte(self.reg.sp, (next_instruction & 0xFF) as u8);

                        cycles = 4;
                        self.reg.pc = n;
                    }
                    RSTAddress::X38 => {
                        let n = 0x0038;
                        self.reg.sp -= 1;
                        self.memory_bus.write_byte(self.reg.sp, (next_instruction >> 8) as u8);
                        self.reg.sp -= 1;
                        self.memory_bus.write_byte(self.reg.sp, (next_instruction & 0xFF) as u8);

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
                        self.reg.a = msb;
                        self.reg.f = lsb;
                    },
                    StackTarget::BC => {
                        let lsb = self.memory_bus.read_byte(self.reg.sp);
                        self.reg.sp += 1;
                        let msb = self.memory_bus.read_byte(self.reg.sp);
                        self.reg.a = msb;
                        self.reg.f = lsb;
                    },
                    StackTarget::DE => {
                        let lsb = self.memory_bus.read_byte(self.reg.sp);
                        self.reg.sp += 1;
                        let msb = self.memory_bus.read_byte(self.reg.sp);
                        self.reg.a = msb;
                        self.reg.f = lsb;
                    },
                    StackTarget::HL => {
                        let lsb = self.memory_bus.read_byte(self.reg.sp);
                        self.reg.sp += 1;
                        let msb = self.memory_bus.read_byte(self.reg.sp);
                        self.reg.a = msb;
                        self.reg.f = lsb;
                    },
                }
                self.reg.sp += 1;

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
                self.reg.pc += 1;
                let prefixed_opcode = self.fetch_prefixed_byte().expect("failed fetching");

                match prefixed_opcode {
                    PrefixedOpcode::RLC(operand) => {
                        match operand {
                            PrefixOperand::A => {
                                let c = (self.reg.a & 0x80) >> 7;
                                let r = self.reg.a.rotate_left(1) | c;

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, c == 0x01);

                                self.reg.a = r;
                            },
                            PrefixOperand::B => {
                                let c = (self.reg.b & 0x80) >> 7;
                                let r = self.reg.b.rotate_left(1) | c;

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, c == 0x01);

                                self.reg.b = r;
                            },
                            PrefixOperand::C => {
                                let c = (self.reg.c & 0x80) >> 7;
                                let r = self.reg.c.rotate_left(1) | c;

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, c == 0x01);

                                self.reg.c = r;
                            },
                            PrefixOperand::D => {
                                let c = (self.reg.d & 0x80) >> 7;
                                let r = self.reg.d.rotate_left(1) | c;

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, c == 0x01);

                                self.reg.d = r;
                            },
                            PrefixOperand::E => {
                                let c = (self.reg.e & 0x80) >> 7;
                                let r = self.reg.e.rotate_left(1) | c;

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, c == 0x01);

                                self.reg.e = r;
                            },
                            PrefixOperand::H => {
                                let c = (self.reg.h & 0x80) >> 7;
                                let r = self.reg.h.rotate_left(1) | c;

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, c == 0x01);

                                self.reg.h = r;
                            },
                            PrefixOperand::L => {
                                let c = (self.reg.l & 0x80) >> 7;
                                let r = self.reg.l.rotate_left(1) | c;

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, c == 0x01);

                                self.reg.l = r;
                            },
                            PrefixOperand::HLIndirect => {
                                let c = (self.reg.c & 0x80) >> 7;
                                let val = self.memory_bus.read_byte(self.reg.hl());
                                let r = val.rotate_left(1) | c;

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, c == 0x01);

                                self.memory_bus.write_byte(self.reg.hl(), r);
                            },
                        }

                        match operand {
                            PrefixOperand::HLIndirect => { cycles = 4; self.reg.pc += 2; },
                            _ => { cycles = 2; self.reg.pc += 1; },
                        }
                    },

                    PrefixedOpcode::RRC(operand) => {
                        match operand {
                            PrefixOperand::A => {
                                let r = self.reg.a.rotate_right(1);

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, self.reg.a & 0x01 == 0x01);

                                self.reg.a = r;
                            },
                            PrefixOperand::B => {
                                let r = self.reg.b.rotate_right(1);

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, self.reg.b & 0x01 == 0x01);

                                self.reg.b = r;
                            },
                            PrefixOperand::C => {
                                let r = self.reg.c.rotate_right(1);

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, self.reg.c & 0x01 == 0x01);

                                self.reg.c = r;
                            },
                            PrefixOperand::D => {
                                let r = self.reg.d.rotate_right(1);

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, self.reg.d & 0x01 == 0x01);

                                self.reg.d = r;
                            },
                            PrefixOperand::E => {
                                let r = self.reg.e.rotate_right(1);

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, self.reg.e & 0x01 == 0x01);

                                self.reg.e = r;
                            },
                            PrefixOperand::H => {
                                let r = self.reg.h.rotate_right(1);

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, self.reg.h & 0x01 == 0x01);

                                self.reg.h = r;
                            },
                            PrefixOperand::L => {
                                let r = self.reg.l.rotate_right(1);

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, self.reg.l & 0x01 == 0x01);

                                self.reg.l = r;
                            },
                            PrefixOperand::HLIndirect => {
                                let val = self.memory_bus.read_byte(self.reg.hl());
                                let r = val.rotate_right(1);

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, val & 0x01 == 0x01);

                                self.memory_bus.write_byte(self.reg.hl(), r);
                            },
                        }

                        match operand {
                            PrefixOperand::HLIndirect => { cycles = 4; self.reg.pc += 2; },
                            _ => { cycles = 2; self.reg.pc += 1; },
                        }
                    },

                    PrefixedOpcode::RL(operand) => {
                        match operand {
                            PrefixOperand::A => {
                                let c = if self.reg.get_flag(Flag::C) { 1 } else { 0 };
                                let r = self.reg.a << 1 | c;
                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, (0x80 & self.reg.a) == 0x80);

                                self.reg.a = r;
                            },
                            PrefixOperand::B => {
                                let c = if self.reg.get_flag(Flag::C) { 1 } else { 0 };
                                let r = self.reg.b << 1 | c;
                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, (0x80 & self.reg.b) == 0x80);

                                self.reg.b = r;
                            },
                            PrefixOperand::C => {
                                let c = if self.reg.get_flag(Flag::C) { 1 } else { 0 };
                                let r = self.reg.c << 1 | c;
                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, (0x80 & self.reg.c) == 0x80);

                                self.reg.c = r;
                            },
                            PrefixOperand::D => {
                                let c = if self.reg.get_flag(Flag::C) { 1 } else { 0 };
                                let r = self.reg.d << 1 | c;
                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, (0x80 & self.reg.d) == 0x80);

                                self.reg.d = r;
                            },
                            PrefixOperand::E => {
                                let c = if self.reg.get_flag(Flag::C) { 1 } else { 0 };
                                let r = self.reg.e << 1 | c;
                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, (0x80 & self.reg.e) == 0x80);

                                self.reg.e = r;
                            },
                            PrefixOperand::H => {
                                let c = if self.reg.get_flag(Flag::C) { 1 } else { 0 };
                                let r = self.reg.h << 1 | c;
                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, (0x80 & self.reg.h) == 0x80);

                                self.reg.h = r;
                            },
                            PrefixOperand::L => {
                                let c = if self.reg.get_flag(Flag::C) { 1 } else { 0 };
                                let r = self.reg.l << 1 | c;
                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, (0x80 & self.reg.l) == 0x80);

                                self.reg.l = r;
                            },
                            PrefixOperand::HLIndirect => {
                                let val = self.memory_bus.read_byte(self.reg.hl());
                                let c = if self.reg.get_flag(Flag::C) { 1 } else { 0 };
                                let r = val << 1 | c;
                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, (0x80 & val) == 0x80);

                                self.memory_bus.write_byte(self.reg.hl(), r);
                            },
                        }

                        match operand {
                            PrefixOperand::HLIndirect => { cycles = 4; self.reg.pc += 2; },
                            _ => { cycles = 2; self.reg.pc += 1; },
                        }
                    },

                    PrefixedOpcode::RR(operand) => {
                        match operand {
                            PrefixOperand::A => {
                                let c = if self.reg.get_flag(Flag::C) { 1 } else { 0 } << 7;
                                let r = c | self.reg.a >> 1;
                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, (0x80 & self.reg.a) == 0x80);

                                self.reg.a = r;
                            },
                            PrefixOperand::B => {
                                let c = if self.reg.get_flag(Flag::C) { 1 } else { 0 } << 7;
                                let r = c | self.reg.b >> 1;
                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, (0x80 & self.reg.b) == 0x80);

                                self.reg.b = r;
                            },
                            PrefixOperand::C => {
                                let c = if self.reg.get_flag(Flag::C) { 1 } else { 0 } << 7;
                                let r = c | self.reg.c >> 1;
                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, (0x80 & self.reg.c) == 0x80);

                                self.reg.c = r;
                            },
                            PrefixOperand::D => {
                                let c = if self.reg.get_flag(Flag::C) { 1 } else { 0 } << 7;
                                let r = c | self.reg.d >> 1;
                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, (0x80 & self.reg.d) == 0x80);

                                self.reg.d = r;
                            },
                            PrefixOperand::E => {
                                let c = if self.reg.get_flag(Flag::C) { 1 } else { 0 } << 7;
                                let r = c | self.reg.e >> 1;
                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, (0x80 & self.reg.e) == 0x80);

                                self.reg.e = r;
                            },
                            PrefixOperand::H => {
                                let c = if self.reg.get_flag(Flag::C) { 1 } else { 0 } << 7;
                                let r = c | self.reg.h >> 1;
                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, (0x80 & self.reg.h) == 0x80);

                                self.reg.h = r;
                            },
                            PrefixOperand::L => {
                                let c = if self.reg.get_flag(Flag::C) { 1 } else { 0 } << 7;
                                let r = c | self.reg.l >> 1;
                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, (0x80 & self.reg.l) == 0x80);

                                self.reg.l = r;
                            },
                            PrefixOperand::HLIndirect => {
                                let val = self.memory_bus.read_byte(self.reg.hl());
                                let c = if self.reg.get_flag(Flag::C) { 1 } else { 0 } << 7;
                                let r = c | val >> 1;

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, (0x80 & val) == 0x80);

                                self.memory_bus.write_byte(self.reg.hl(), r);
                            },
                        }

                        match operand {
                            PrefixOperand::HLIndirect => { cycles = 4; self.reg.pc += 2; },
                            _ => { cycles = 2; self.reg.pc += 1; },
                        }
                    },

                    PrefixedOpcode::SLA(operand) => {
                        match operand {
                            PrefixOperand::A => {
                                let r = self.reg.a << 1;

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, (0x80 & self.reg.a) == 0x80);

                                self.reg.a = r;
                            },
                            PrefixOperand::B => {
                                let r = self.reg.b << 1;

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, (0x80 & self.reg.b) == 0x80);

                                self.reg.b = r;
                            },
                            PrefixOperand::C => {
                                let r = self.reg.c << 1;

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, (0x80 & self.reg.c) == 0x80);

                                self.reg.c = r;
                            },
                            PrefixOperand::D => {
                                let r = self.reg.d << 1;

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, (0x80 & self.reg.d) == 0x80);

                                self.reg.d = r;
                            },
                            PrefixOperand::E => {
                                let r = self.reg.e << 1;

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, (0x80 & self.reg.e) == 0x80);

                                self.reg.e = r;
                            },
                            PrefixOperand::H => {
                                let r = self.reg.h << 1;

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, (0x80 & self.reg.h) == 0x80);

                                self.reg.h = r;
                            },
                            PrefixOperand::L => {
                                let r = self.reg.l << 1;

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, (0x80 & self.reg.l) == 0x80);

                                self.reg.l = r;
                            },
                            PrefixOperand::HLIndirect => {
                                let val = self.memory_bus.read_byte(self.reg.hl());
                                let r = val << 1;

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, (0x80 & val) == 0x80);

                                self.memory_bus.write_byte(self.reg.hl(), r);
                            },
                        }

                        match operand {
                            PrefixOperand::HLIndirect => { cycles = 4; self.reg.pc += 2; },
                            _ => { cycles = 2; self.reg.pc += 1; },
                        }
                    },
                    PrefixedOpcode::SRA(operand) => {
                        match operand {
                            PrefixOperand::A => {
                                let sign_bit = self.reg.a & 0x80;
                                let r = sign_bit | self.reg.a >> 1;

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, (0x01 & self.reg.a) == 0x01);

                                self.reg.a = r;
                            },
                            PrefixOperand::B => {
                                let sign_bit = self.reg.b & 0x80;
                                let r = sign_bit | self.reg.b >> 1;

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, (0x01 & self.reg.b) == 0x01);

                                self.reg.b = r;
                            },
                            PrefixOperand::C => {
                                let sign_bit = self.reg.c & 0x80;
                                let r = sign_bit | self.reg.c >> 1;

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, (0x01 & self.reg.c) == 0x01);

                                self.reg.c = r;
                            },
                            PrefixOperand::D => {
                                let sign_bit = self.reg.d & 0x80;
                                let r = sign_bit | self.reg.d >> 1;

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, (0x01 & self.reg.d) == 0x01);

                                self.reg.d = r;
                            },
                            PrefixOperand::E => {
                                let sign_bit = self.reg.e & 0x80;
                                let r = sign_bit | self.reg.e >> 1;

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, (0x01 & self.reg.e) == 0x01);

                                self.reg.e = r;
                            },
                            PrefixOperand::H => {
                                let sign_bit = self.reg.h & 0x80;
                                let r = sign_bit | self.reg.h >> 1;

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, (0x01 & self.reg.h) == 0x01);

                                self.reg.h = r;
                            },
                            PrefixOperand::L => {
                                let sign_bit = self.reg.l & 0x80;
                                let r = sign_bit | self.reg.l >> 1;

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, (0x01 & self.reg.l) == 0x01);

                                self.reg.l = r;
                            },
                            PrefixOperand::HLIndirect => {
                                let val = self.memory_bus.read_byte(self.reg.hl());
                                let sign_bit = val & 0x80;
                                let r = sign_bit | val >> 1;

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, (0x01 & val) == 0x01);

                                self.memory_bus.write_byte(self.reg.hl(), r);
                            },
                        }

                        match operand {
                            PrefixOperand::HLIndirect => { cycles = 4; self.reg.pc += 2; },
                            _ => { cycles = 2; self.reg.pc += 1; },
                        }
                    },
                    PrefixedOpcode::SRL(operand) => {
                        match operand {
                            PrefixOperand::A => {
                                let r = self.reg.a >> 1;

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, (0x01 & self.reg.a) == 0x01);

                                self.reg.a = r;
                            },
                            PrefixOperand::B => {
                                let r = self.reg.b >> 1;

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, (0x01 & self.reg.b) == 0x01);

                                self.reg.b = r;
                            },
                            PrefixOperand::C => {
                                let r = self.reg.c >> 1;

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, (0x01 & self.reg.c) == 0x01);

                                self.reg.c = r;
                            },
                            PrefixOperand::D => {
                                let r = self.reg.d >> 1;

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, (0x01 & self.reg.d) == 0x01);

                                self.reg.d = r;
                            },
                            PrefixOperand::E => {
                                let r = self.reg.e >> 1;

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, (0x01 & self.reg.e) == 0x01);

                                self.reg.e = r;
                            },
                            PrefixOperand::H => {
                                let r = self.reg.h >> 1;

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, (0x01 & self.reg.h) == 0x01);

                                self.reg.h = r;
                            },
                            PrefixOperand::L => {
                                let r = self.reg.l >> 1;

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, (0x01 & self.reg.l) == 0x01);

                                self.reg.l = r;
                            },
                            PrefixOperand::HLIndirect => {
                                let val = self.memory_bus.read_byte(self.reg.hl());
                                let r = val >> 1;

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, (0x01 & val) == 0x01);

                                self.memory_bus.write_byte(self.reg.hl(), r);
                            },
                        }

                        match operand {
                            PrefixOperand::HLIndirect => { cycles = 4; self.reg.pc += 2; },
                            _ => { cycles = 2; self.reg.pc += 1; },
                        }
                    },

                    PrefixedOpcode::SWAP(operand) => {
                        match operand {
                            PrefixOperand::A => {
                                let r = (self.reg.a << 4) | (self.reg.a >> 4);

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, false);

                                self.reg.a = r;
                            },
                            PrefixOperand::B => {
                                let r = (self.reg.b << 4) | (self.reg.b >> 4);

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, false);

                                self.reg.b = r;
                            },
                            PrefixOperand::C => {
                                let r = (self.reg.c << 4) | (self.reg.c >> 4);

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, false);

                                self.reg.c = r;
                            },
                            PrefixOperand::D => {
                                let r = (self.reg.d << 4) | (self.reg.d >> 4);

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, false);

                                self.reg.d = r;
                            },
                            PrefixOperand::E => {
                                let r = (self.reg.e << 4) | (self.reg.e >> 4);

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, false);

                                self.reg.e = r;
                            },
                            PrefixOperand::H => {
                                let r = (self.reg.h << 4) | (self.reg.h >> 4);

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, false);

                                self.reg.h = r;
                            },
                            PrefixOperand::L => {
                                let r = (self.reg.l << 4) | (self.reg.l >> 4);

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, false);

                                self.reg.l = r;
                            },
                            PrefixOperand::HLIndirect => {
                                let val = self.memory_bus.read_byte(self.reg.hl());

                                let r = (val << 4) | (val >> 4);

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, false);
                                self.reg.set_flag(Flag::C, false);

                                self.memory_bus.write_byte(self.reg.hl(), r);
                            },
                        }

                        match operand {
                            PrefixOperand::HLIndirect => { cycles = 4; self.reg.pc += 2; },
                            _ => { cycles = 2; self.reg.pc += 1; },
                        }
                    }
                    PrefixedOpcode::BIT(operand, position) => {
                        let &reg;
                        let hl_byte = self.memory_bus.read_byte(self.reg.hl());

                        match operand {
                            PrefixOperand::A => {
                                reg = &self.reg.a;
                            },
                            PrefixOperand::B => {
                                reg = &self.reg.b;
                            },
                            PrefixOperand::C => {
                                reg = &self.reg.c;
                            },
                            PrefixOperand::D => {
                                reg = &self.reg.d;
                            },
                            PrefixOperand::E => {
                                reg = &self.reg.e;
                            },
                            PrefixOperand::H => {
                                reg = &self.reg.h;
                            },
                            PrefixOperand::L => {
                                reg = &self.reg.l;
                            },
                            PrefixOperand::HLIndirect => {
                                reg = &hl_byte;
                            },
                        }

                        match position {
                            BitPosition::B0 => {
                                let r = *reg & 0b1;

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, true);
                            },
                            BitPosition::B1 => {
                                let r = (*reg >> 1) & 0b1;

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, true);
                            },
                            BitPosition::B2 => {
                                let r = (*reg >> 2) & 0b1;

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, true);
                            },
                            BitPosition::B3 => {
                                let r = (*reg >> 3) & 0b1;

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, true);
                            },
                            BitPosition::B4 => {
                                let r = (*reg >> 4) & 0b1;

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, true);
                            },
                            BitPosition::B5 => {
                                let r = (*reg >> 5) & 0b1;

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, true);
                            },
                            BitPosition::B6 => {
                                let r = (*reg >> 6) & 0b1;

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, true);
                            },
                            BitPosition::B7 => {
                                let r = (*reg >> 7) & 0b1;

                                self.reg.set_flag(Flag::Z, r == 0);
                                self.reg.set_flag(Flag::N, false);
                                self.reg.set_flag(Flag::H, true);
                            },
                        }

                        match operand {
                            PrefixOperand::HLIndirect => { cycles = 4; self.reg.pc += 2; },
                            _ => { cycles = 2; self.reg.pc += 1; },
                        }
                    }
                    PrefixedOpcode::RES(operand, position) => {
                        let reg: &mut u8;
                        let mut hl_byte = self.memory_bus.read_byte(self.reg.hl());

                        match operand {
                            PrefixOperand::A => {
                                reg = &mut self.reg.a;
                            },
                            PrefixOperand::B => {
                                reg = &mut self.reg.b;
                            },
                            PrefixOperand::C => {
                                reg = &mut self.reg.c;
                            },
                            PrefixOperand::D => {
                                reg = &mut self.reg.d;
                            },
                            PrefixOperand::E => {
                                reg = &mut self.reg.e;
                            },
                            PrefixOperand::H => {
                                reg = &mut self.reg.h;
                            },
                            PrefixOperand::L => {
                                reg = &mut self.reg.l;
                            },
                            PrefixOperand::HLIndirect => {
                                reg = &mut hl_byte;
                            },
                        }

                        match position {
                            BitPosition::B0 => {
                                *reg = *reg % !0b1;
                            },
                            BitPosition::B1 => {
                                *reg = *reg & !(0b1 << 1);
                            },
                            BitPosition::B2 => {
                                *reg = *reg & !(0b1 << 2);
                            },
                            BitPosition::B3 => {
                                *reg = *reg & !(0b1 << 3);
                            },
                            BitPosition::B4 => {
                                *reg = *reg & !(0b1 << 4);
                            },
                            BitPosition::B5 => {
                                *reg = *reg & !(0b1 << 5);
                            },
                            BitPosition::B6 => {
                                *reg = *reg & !(0b1 << 6);
                            },
                            BitPosition::B7 => {
                                *reg = *reg & !(0b1 << 7);
                            },
                        }

                        match operand {
                            PrefixOperand::HLIndirect => { cycles = 4; self.reg.pc += 2; },
                            _ => { cycles = 2; self.reg.pc += 1; },
                        }
                    },
                    PrefixedOpcode::SET(operand, position) => {
                        let reg: &mut u8;
                        let mut hl_byte = self.memory_bus.read_byte(self.reg.hl());

                        match operand {
                            PrefixOperand::A => {
                                reg = &mut self.reg.a;
                            },
                            PrefixOperand::B => {
                                reg = &mut self.reg.b;
                            },
                            PrefixOperand::C => {
                                reg = &mut self.reg.c;
                            },
                            PrefixOperand::D => {
                                reg = &mut self.reg.d;
                            },
                            PrefixOperand::E => {
                                reg = &mut self.reg.e;
                            },
                            PrefixOperand::H => {
                                reg = &mut self.reg.h;
                            },
                            PrefixOperand::L => {
                                reg = &mut self.reg.l;
                            },
                            PrefixOperand::HLIndirect => {
                                reg = &mut hl_byte;
                            },
                        }

                        match position {
                            BitPosition::B0 => {
                                *reg = *reg | 0b1;
                            },
                            BitPosition::B1 => {
                                *reg = *reg | (0b1 << 1);
                            },
                            BitPosition::B2 => {
                                *reg = *reg | (0b1 << 2);
                            },
                            BitPosition::B3 => {
                                *reg = *reg | (0b1 << 3);
                            },
                            BitPosition::B4 => {
                                *reg = *reg | (0b1 << 4);
                            },
                            BitPosition::B5 => {
                                *reg = *reg | (0b1 << 5);
                            },
                            BitPosition::B6 => {
                                *reg = *reg | (0b1 << 6);
                            },
                            BitPosition::B7 => {
                                *reg = *reg | (0b1 << 7);
                            },
                        }

                        match operand {
                            PrefixOperand::HLIndirect => { cycles = 4; self.reg.pc += 2; },
                            _ => { cycles = 2; self.reg.pc += 1; },
                        }
                    },

                }
            }
        };

        self.log_debug(format!("{} cycles", cycles));
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

        println!("cycles_to_run: {}", cycles_to_run);

        loop {
            self.log_debug(format!("emulating..."));

            let cycles = self.execute();

            cycles_to_run -= cycles as i32;
            cycles_ran += cycles as usize;

            if cycles_to_run <= 0 {
                break;
            }
        }

        cycles_ran
    }
}
