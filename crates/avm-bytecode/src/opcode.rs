//! Instruction set per TZ §3.2.2.

use serde::{Deserialize, Serialize};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Opcode {
    Push = 0,
    Pop = 1,
    Dup = 2,
    Swap = 3,
    Add = 4,
    Sub = 5,
    Mul = 6,
    Div = 7,
    Mod = 8,
    Neg = 9,
    Eq = 10,
    Ne = 11,
    Lt = 12,
    Gt = 13,
    Le = 14,
    Ge = 15,
    And = 16,
    Or = 17,
    Not = 18,
    Jmp = 19,
    JmpIf = 20,
    JmpIfNot = 21,
    Call = 22,
    Ret = 23,
    GetField = 24,
    SetField = 25,
    ArrayGet = 26,
    ArraySet = 27,
    ArrayLen = 28,
    NewStruct = 29,
    NewArray = 30,
    NewMap = 31,
    Emit = 32,
    NextEvent = 33,
    Predict = 34,
    Serialize = 35,
    Deserialize = 36,
    LoadLocal = 37,
    StoreLocal = 38,
    Return = 39,
    Halt = 255,
}

impl Opcode {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Push),
            1 => Some(Self::Pop),
            2 => Some(Self::Dup),
            3 => Some(Self::Swap),
            4 => Some(Self::Add),
            5 => Some(Self::Sub),
            6 => Some(Self::Mul),
            7 => Some(Self::Div),
            8 => Some(Self::Mod),
            9 => Some(Self::Neg),
            10 => Some(Self::Eq),
            11 => Some(Self::Ne),
            12 => Some(Self::Lt),
            13 => Some(Self::Gt),
            14 => Some(Self::Le),
            15 => Some(Self::Ge),
            16 => Some(Self::And),
            17 => Some(Self::Or),
            18 => Some(Self::Not),
            19 => Some(Self::Jmp),
            20 => Some(Self::JmpIf),
            21 => Some(Self::JmpIfNot),
            22 => Some(Self::Call),
            23 => Some(Self::Ret),
            24 => Some(Self::GetField),
            25 => Some(Self::SetField),
            26 => Some(Self::ArrayGet),
            27 => Some(Self::ArraySet),
            28 => Some(Self::ArrayLen),
            29 => Some(Self::NewStruct),
            30 => Some(Self::NewArray),
            31 => Some(Self::NewMap),
            32 => Some(Self::Emit),
            33 => Some(Self::NextEvent),
            34 => Some(Self::Predict),
            35 => Some(Self::Serialize),
            36 => Some(Self::Deserialize),
            37 => Some(Self::LoadLocal),
            38 => Some(Self::StoreLocal),
            39 => Some(Self::Return),
            255 => Some(Self::Halt),
            _ => None,
        }
    }

    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Operand {
    I64(i64),
    F64(f64),
    Bool(bool),
    Str(String),
    U32(u32),
    U16(u16),
    U8(u8),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Instruction {
    pub op: Opcode,
    pub operand: Option<Operand>,
}

impl Instruction {
    pub fn new(op: Opcode) -> Self {
        Self { op, operand: None }
    }

    pub fn with_operand(op: Opcode, operand: Operand) -> Self {
        Self {
            op,
            operand: Some(operand),
        }
    }
}
