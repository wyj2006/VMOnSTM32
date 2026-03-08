use crate::{cpu::CPU, vm_exception::VMException};
use bitvec::{
    field::BitField, order::Lsb0, slice::BitSlice, store::BitStore, vec::BitVec, view::BitView,
};
use funty::Integral;

pub static OPERAND_MAX_NUM: usize = 3;

#[derive(Clone, Copy)]
pub enum Opcode {
    Auipc,
    Branch(Condition),

    CSRClear,
    CSRSet,
    CSRWrite,

    FAdd(DataType),
    FClassify(DataType),
    FCompare(Condition, DataType),
    // from, to
    FConvert(DataType, DataType),
    FDiv(DataType),
    FLoad(DataType),
    FMax(DataType),
    FMin(DataType),
    // from, to
    FMove(DataType, DataType),
    FMul(DataType),
    FSignJoin(SignJoinKind, DataType),
    FSqrt(DataType),
    FStore(DataType),
    FSub(DataType),

    IAdd,
    IAnd,
    //keep_rem, is_unsigned
    IDiv(bool, bool),
    // data_width, is_unsigned
    ILoad(DataWidth, bool),
    // keep_high, a is_unsigned, b is_unsigned
    IMul(bool, bool, bool),
    IStore(DataWidth),
    ISub,

    Jal,
    Jalr,
    Lui,
    MRet,
    Or,
    SFenceVMA,
    Sll,
    // is_unsigned
    Slt(bool),
    Sra,
    SRet,
    Srl,
    Xor,
    Wfi,
}

#[derive(Clone, Copy)]
pub enum SignJoinKind {
    Default,
    Negative,
    Xor,
}

#[derive(Clone, Copy)]
pub enum Condition {
    Eq,
    Neq,
    Lt,
    Le,
    Ge,
    Ltu,
    Geu,
}

#[derive(Clone, Copy)]
pub enum DataWidth {
    Byte,
    HalfWord,
    Word,
}

#[derive(Clone, Copy)]
pub enum DataType {
    I32,
    Float,
    Double,
}

#[derive(Clone, Copy)]
pub enum Operand {
    XReg(usize),
    FReg(usize),
    DReg(usize),
    Imm(u32),
    Csr(u32),
    Nothing,
}

pub struct Instruction {
    pub opcode: Opcode,
    pub operand: [Operand; OPERAND_MAX_NUM],
}

impl CPU {
    pub fn decode(&self, inst: u32) -> Result<Instruction, VMException> {
        let inst = inst.view_bits::<Lsb0>();

        let opcode = inst.get(0..=6).unwrap().load::<u32>();
        let rd = inst.get(7..=11).unwrap().load::<usize>();
        let rs1 = inst.get(15..=19).unwrap().load::<usize>();
        let rs2 = inst.get(20..=24).unwrap().load::<usize>();
        let func3 = inst.get(12..=14).unwrap().load::<u8>();
        let func7 = inst.get(25..=31).unwrap().load::<u8>();
        let imm = signed(inst, 20, 31);

        match opcode {
            0b0000011 => {
                let (data_width, is_unsigned) = match func3 {
                    0b000 => (DataWidth::Byte, false),
                    0b001 => (DataWidth::HalfWord, false),
                    0b010 => (DataWidth::Word, false),
                    0b100 => (DataWidth::Byte, true),
                    0b101 => (DataWidth::HalfWord, true),
                    _ => return Err(VMException::IllegalInstruction),
                };
                Ok(Instruction {
                    opcode: Opcode::ILoad(data_width, is_unsigned),
                    operand: [Operand::XReg(rd), Operand::XReg(rs1), Operand::Imm(imm)],
                })
            }
            0b0000111 => match func3 {
                0b010 => Ok(Instruction {
                    opcode: Opcode::FLoad(DataType::Float),
                    operand: [Operand::FReg(rd), Operand::XReg(rs1), Operand::Imm(imm)],
                }),
                0b011 => Ok(Instruction {
                    opcode: Opcode::FLoad(DataType::Double),
                    operand: [Operand::DReg(rd), Operand::XReg(rs1), Operand::Imm(imm)],
                }),
                _ => return Err(VMException::IllegalInstruction),
            },
            0b0010011 => {
                let opcode = match func3 {
                    0b000 => Opcode::IAdd,
                    0b001 => Opcode::Sll,
                    0b010 => Opcode::Slt(false),
                    0b011 => Opcode::Slt(true),
                    0b100 => Opcode::Xor,
                    0b101 => match func7 {
                        0b0000000 => Opcode::Srl,
                        0b0100000 => Opcode::Sra,
                        _ => return Err(VMException::IllegalInstruction),
                    },
                    0b110 => Opcode::Or,
                    0b111 => Opcode::IAnd,
                    _ => return Err(VMException::IllegalInstruction),
                };

                let operand = match opcode {
                    //shamt
                    Opcode::Sll | Opcode::Srl | Opcode::Sra => {
                        Operand::Imm(signed_extend(rs2 as u32, 5))
                    }
                    _ => Operand::Imm(imm),
                };

                Ok(Instruction {
                    opcode,
                    operand: [Operand::XReg(rd), Operand::XReg(rs1), operand],
                })
            }
            0b0010111 => {
                let mut imm: BitVec<_, Lsb0> = BitVec::from_element(0u32);
                imm[12..=31].store(inst.get(12..=31).unwrap().load::<u32>());
                let imm = imm.load::<u32>();

                Ok(Instruction {
                    opcode: Opcode::Auipc,
                    operand: [Operand::XReg(rd), Operand::Imm(imm), Operand::Nothing],
                })
            }
            0b0100011 => {
                let mut imm: BitVec<_, Lsb0> = BitVec::from_element(0u32);
                imm[0..=4].store(inst.get(7..=11).unwrap().load::<u32>());
                imm[5..=11].store(inst.get(25..=31).unwrap().load::<u32>());
                let imm = signed_extend(imm.load::<u32>(), 11 + 1);

                Ok(Instruction {
                    opcode: Opcode::IStore(match func3 {
                        0b000 => DataWidth::Byte,
                        0b001 => DataWidth::HalfWord,
                        0b010 => DataWidth::Word,
                        _ => return Err(VMException::IllegalInstruction),
                    }),
                    operand: [Operand::XReg(rs1), Operand::XReg(rs2), Operand::Imm(imm)],
                })
            }
            0b0100111 => {
                let mut imm: BitVec<_, Lsb0> = BitVec::from_element(0u32);
                imm[0..=4].store(inst.get(7..=11).unwrap().load::<u32>());
                imm[5..=11].store(inst.get(25..=31).unwrap().load::<u32>());
                let imm = signed_extend(imm.load::<u32>(), 11 + 1);

                match func3 {
                    0b010 => Ok(Instruction {
                        opcode: Opcode::FStore(DataType::Float),
                        operand: [Operand::XReg(rs1), Operand::FReg(rs2), Operand::Imm(imm)],
                    }),
                    0b011 => Ok(Instruction {
                        opcode: Opcode::FStore(DataType::Double),
                        operand: [Operand::XReg(rs1), Operand::DReg(rs2), Operand::Imm(imm)],
                    }),
                    _ => return Err(VMException::IllegalInstruction),
                }
            }
            0b0110011 => match func7 {
                0b0000001 => Ok(Instruction {
                    opcode: match func3 {
                        0b000 => Opcode::IMul(false, false, false),
                        0b001 => Opcode::IMul(true, false, false),
                        0b010 => Opcode::IMul(true, false, true),
                        0b011 => Opcode::IMul(true, true, true),
                        0b100 => Opcode::IDiv(false, false),
                        0b101 => Opcode::IDiv(false, true),
                        0b110 => Opcode::IDiv(true, false),
                        0b111 => Opcode::IDiv(true, true),
                        _ => return Err(VMException::IllegalInstruction),
                    },
                    operand: [Operand::XReg(rd), Operand::XReg(rs1), Operand::XReg(rs2)],
                }),
                _ => Ok(Instruction {
                    opcode: match func3 {
                        0b000 => match func7 {
                            0b0000000 => Opcode::IAdd,
                            0b0100000 => Opcode::ISub,
                            _ => return Err(VMException::IllegalInstruction),
                        },
                        0b001 => Opcode::Sll,
                        0b010 => Opcode::Slt(false),
                        0b011 => Opcode::Slt(true),
                        0b100 => Opcode::Xor,
                        0b101 => match func7 {
                            0b0000000 => Opcode::Srl,
                            0b0100000 => Opcode::Sra,
                            _ => return Err(VMException::IllegalInstruction),
                        },
                        0b110 => Opcode::Or,
                        0b111 => Opcode::IAnd,
                        _ => return Err(VMException::IllegalInstruction),
                    },
                    operand: [Operand::XReg(rd), Operand::XReg(rs1), Operand::XReg(rs2)],
                }),
            },
            0b0110111 => {
                let mut imm: BitVec<_, Lsb0> = BitVec::from_element(0u32);
                imm[12..=31].store(inst.get(12..=31).unwrap().load::<u32>());
                let imm = imm.load::<u32>();

                Ok(Instruction {
                    opcode: Opcode::Lui,
                    operand: [Operand::XReg(rd), Operand::Imm(imm), Operand::Nothing],
                })
            }
            0b1010011 => {
                //TODO let rm = func3 as usize;
                match rs2 {
                    0 => {
                        let opcode = match func7 {
                            0b0001100 => Opcode::FSqrt(DataType::Float),
                            0b0001101 => Opcode::FSqrt(DataType::Double),
                            0b0100001 => Opcode::FConvert(DataType::Float, DataType::Double),
                            0b1100000 => Opcode::FConvert(DataType::Float, DataType::I32),
                            0b1100001 => Opcode::FConvert(DataType::Double, DataType::I32),
                            0b1101000 => Opcode::FConvert(DataType::I32, DataType::Float),
                            0b1101001 => Opcode::FMove(DataType::I32, DataType::Double),
                            0b1110000 => match func3 {
                                0b000 => Opcode::FMove(DataType::I32, DataType::Float),
                                0b001 => Opcode::FClassify(DataType::Float),
                                _ => return Err(VMException::IllegalInstruction),
                            },
                            0b1110001 => Opcode::FClassify(DataType::Double),
                            0b1111000 => Opcode::FMove(DataType::Float, DataType::I32),
                            _ => return Err(VMException::IllegalInstruction),
                        };
                        Ok(Instruction {
                            opcode,
                            operand: [
                                match opcode {
                                    Opcode::FClassify(..)
                                    | Opcode::FConvert(_, DataType::I32)
                                    | Opcode::FMove(_, DataType::I32) => Operand::XReg(rd),
                                    Opcode::FSqrt(DataType::Double)
                                    | Opcode::FConvert(_, DataType::Double)
                                    | Opcode::FMove(_, DataType::Double) => Operand::DReg(rd),
                                    _ => Operand::FReg(rd),
                                },
                                match opcode {
                                    Opcode::FConvert(DataType::I32, _)
                                    | Opcode::FMove(DataType::I32, _) => Operand::XReg(rs1),
                                    Opcode::FConvert(DataType::Double, _)
                                    | Opcode::FMove(DataType::Double, _) => Operand::DReg(rs1),
                                    _ => Operand::FReg(rs1),
                                },
                                Operand::Nothing,
                            ],
                        })
                    }
                    1 => Ok(Instruction {
                        opcode: Opcode::FConvert(DataType::Double, DataType::Float),
                        operand: [Operand::FReg(rd), Operand::DReg(rs1), Operand::Nothing],
                    }),
                    _ => {
                        let data_type = if func7 & 1 == 0 {
                            DataType::Float
                        } else {
                            DataType::Double
                        };

                        let opcode = match func7 >> 1 {
                            0b000000 => Opcode::FAdd(data_type),
                            0b000010 => Opcode::FSub(data_type),
                            0b000100 => Opcode::FMul(data_type),
                            0b000110 => Opcode::FDiv(data_type),
                            0b001000 => match func3 {
                                0b000 => Opcode::FSignJoin(SignJoinKind::Default, data_type),
                                0b001 => Opcode::FSignJoin(SignJoinKind::Negative, data_type),
                                0b010 => Opcode::FSignJoin(SignJoinKind::Xor, data_type),
                                _ => return Err(VMException::IllegalInstruction),
                            },
                            0b001010 => match func3 {
                                0b000 => Opcode::FMin(data_type),
                                0b001 => Opcode::FMax(data_type),
                                _ => return Err(VMException::IllegalInstruction),
                            },
                            0b101000 => match func3 {
                                0b000 => Opcode::FCompare(Condition::Le, data_type),
                                0b001 => Opcode::FCompare(Condition::Lt, data_type),
                                0b010 => Opcode::FCompare(Condition::Eq, data_type),
                                _ => return Err(VMException::IllegalInstruction),
                            },
                            _ => return Err(VMException::IllegalInstruction),
                        };
                        Ok(Instruction {
                            opcode,
                            operand: [
                                match (opcode, data_type) {
                                    (Opcode::FCompare(..), _) => Operand::XReg(rd),
                                    (_, DataType::Double) => Operand::DReg(rd),
                                    (_, DataType::Float) => Operand::FReg(rd),
                                    _ => unreachable!(),
                                },
                                match data_type {
                                    DataType::Double => Operand::DReg(rs1),
                                    DataType::Float => Operand::FReg(rs1),
                                    _ => unreachable!(),
                                },
                                match data_type {
                                    DataType::Double => Operand::DReg(rs2),
                                    DataType::Float => Operand::FReg(rs2),
                                    _ => unreachable!(),
                                },
                            ],
                        })
                    }
                }
            }
            0b1100011 => {
                let mut imm: BitVec<_, Lsb0> = BitVec::from_element(0u32);
                imm[11..=11].store(inst.get(7..=7).unwrap().load::<u32>());
                imm[1..=4].store(inst.get(8..=11).unwrap().load::<u32>());
                imm[5..=10].store(inst.get(25..=30).unwrap().load::<u32>());
                imm[12..=12].store(inst.get(31..=31).unwrap().load::<u32>());
                let imm = signed_extend(imm.load::<u32>(), 12 + 1);

                Ok(Instruction {
                    opcode: Opcode::Branch(match func3 {
                        0b000 => Condition::Eq,
                        0b001 => Condition::Neq,
                        0b100 => Condition::Lt,
                        0b101 => Condition::Ge,
                        0b110 => Condition::Ltu,
                        0b111 => Condition::Geu,
                        _ => return Err(VMException::IllegalInstruction),
                    }),
                    operand: [Operand::XReg(rs1), Operand::XReg(rs2), Operand::Imm(imm)],
                })
            }
            0b1100111 => Ok(Instruction {
                opcode: Opcode::Jalr,
                operand: [Operand::XReg(rd), Operand::XReg(rs1), Operand::Imm(imm)],
            }),
            0b1101111 => {
                let mut imm: BitVec<_, Lsb0> = BitVec::from_element(0u32);
                imm[12..=19].store(inst.get(12..=19).unwrap().load::<u32>());
                imm[11..=11].store(inst.get(20..=20).unwrap().load::<u32>());
                imm[1..=10].store(inst.get(21..=30).unwrap().load::<u32>());
                imm[20..=20].store(inst.get(31..=31).unwrap().load::<u32>());
                let imm = signed_extend(imm.load::<u32>(), 20 + 1);

                Ok(Instruction {
                    opcode: Opcode::Jal,
                    operand: [Operand::XReg(rd), Operand::Imm(imm), Operand::Nothing],
                })
            }
            0b1110011 => match func7 {
                0b0001000 => match rs2 {
                    0b00010 => Ok(Instruction {
                        opcode: Opcode::SRet,
                        operand: [Operand::Nothing, Operand::Nothing, Operand::Nothing],
                    }),
                    0b00101 => Ok(Instruction {
                        opcode: Opcode::Wfi,
                        operand: [Operand::Nothing, Operand::Nothing, Operand::Nothing],
                    }),
                    _ => unreachable!(),
                },
                0b0001001 => Ok(Instruction {
                    opcode: Opcode::SFenceVMA,
                    operand: [Operand::XReg(rs1), Operand::XReg(rs2), Operand::Nothing],
                }),
                0b0011000 => Ok(Instruction {
                    opcode: Opcode::MRet,
                    operand: [Operand::Nothing, Operand::Nothing, Operand::Nothing],
                }),
                _ => {
                    let csr = inst.get(20..=31).unwrap().load();
                    let (opcode, operand) = match func3 {
                        0b001 => (Opcode::CSRWrite, Operand::XReg(rs1)),
                        0b010 => (Opcode::CSRSet, Operand::XReg(rs1)),
                        0b011 => (Opcode::CSRClear, Operand::XReg(rs1)),
                        0b101 => (Opcode::CSRWrite, Operand::Imm(rs1 as u32)),
                        0b110 => (Opcode::CSRSet, Operand::Imm(rs1 as u32)),
                        0b111 => (Opcode::CSRClear, Operand::Imm(rs1 as u32)),
                        _ => unreachable!(),
                    };
                    Ok(Instruction {
                        opcode,
                        operand: [Operand::XReg(rd), Operand::Csr(csr), operand],
                    })
                }
            },
            _ => Err(VMException::IllegalInstruction),
        }
    }
}

pub fn signed<'a, T>(value: &'a BitSlice<T, Lsb0>, start: usize, end: usize) -> u32
where
    T: Integral + BitStore,
{
    signed_extend(
        value.get(start..=end).unwrap().load::<u32>(),
        (end - start + 1) as u32,
    )
}

pub fn signed_extend(value: u32, width: u32) -> u32 {
    ((value as i32) << (i32::BITS - width) >> (i32::BITS - width)) as u32
}
