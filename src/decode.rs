use crate::{cpu::CPU, exception::Exception};
use bitvec::{
    field::BitField, order::Lsb0, slice::BitSlice, store::BitStore, vec::BitVec, view::BitView,
};
use funty::Integral;

pub static OPERAND_MAX_NUM: usize = 3;

pub enum Opcode {
    Lui,
    Auipc,
    Jal,
    Jalr,
    Branch(Condition),
    // data_width, is_unsigned
    Load(DataWidth, bool),
    Store(DataWidth),
    Add,
    Sub,
    // is_unsigned
    Slt(bool),
    Xor,
    Or,
    And,
    Sll,
    Srl,
    Sra,
    // keep_high, a is_unsigned, b is_unsigned
    Mul(bool, bool, bool),
    //keep_rem, is_unsigned
    Div(bool, bool),
    CSRWrite,
    CSRSet,
    CSRClear,
    SRet,
    MRet,
    Wfi,
}

pub enum Condition {
    Eq,
    Neq,
    Lt,
    Ge,
    Ltu,
    Geu,
}

pub enum DataWidth {
    Byte,
    HalfWord,
    Word,
}

#[derive(Clone, Copy)]
pub enum Operand {
    Reg(usize),
    Imm(u32),
    Csr(u32),
    Nothing,
}

pub struct Instruction {
    pub opcode: Opcode,
    pub operand: [Operand; OPERAND_MAX_NUM],
}

impl CPU {
    pub fn decode(&self, inst: u32) -> Result<Instruction, Exception> {
        let inst = inst.view_bits::<Lsb0>();

        let opcode = inst.get(0..=6).unwrap().load::<u32>();
        let rd = inst.get(7..=11).unwrap().load::<usize>();
        let rs1 = inst.get(15..=19).unwrap().load::<usize>();
        let rs2 = inst.get(20..=24).unwrap().load::<usize>();
        let func3 = inst.get(12..=14).unwrap().load::<u8>();
        let func7 = inst.get(25..=31).unwrap().load::<u8>();
        let imm = signed(inst, 20, 31);

        match opcode {
            0b0110111 => {
                let mut imm: BitVec<_, Lsb0> = BitVec::from_element(0u32);
                imm[12..=31].store(inst.get(12..=31).unwrap().load::<u32>());
                let imm = imm.load::<u32>();

                Ok(Instruction {
                    opcode: Opcode::Lui,
                    operand: [Operand::Reg(rd), Operand::Imm(imm), Operand::Nothing],
                })
            }
            0b0010111 => {
                let mut imm: BitVec<_, Lsb0> = BitVec::from_element(0u32);
                imm[12..=31].store(inst.get(12..=31).unwrap().load::<u32>());
                let imm = imm.load::<u32>();

                Ok(Instruction {
                    opcode: Opcode::Auipc,
                    operand: [Operand::Reg(rd), Operand::Imm(imm), Operand::Nothing],
                })
            }
            0b1101111 => {
                let mut imm: BitVec<_, Lsb0> = BitVec::from_element(0u32);
                imm[12..=19].store(inst.get(12..=19).unwrap().load::<u32>());
                imm[11..=11].store(inst.get(20..=20).unwrap().load::<u32>());
                imm[1..=10].store(inst.get(21..=30).unwrap().load::<u32>());
                imm[20..=20].store(inst.get(31..=31).unwrap().load::<u32>());
                let imm = signed_extend(imm.load::<u32>(), 20 + 1);

                Ok(Instruction {
                    opcode: Opcode::Jal,
                    operand: [Operand::Reg(rd), Operand::Imm(imm), Operand::Nothing],
                })
            }
            0b1100111 => Ok(Instruction {
                opcode: Opcode::Jalr,
                operand: [Operand::Reg(rd), Operand::Reg(rs1), Operand::Imm(imm)],
            }),
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
                        _ => return Err(Exception::IllegalInstruction),
                    }),
                    operand: [Operand::Reg(rs1), Operand::Reg(rs2), Operand::Imm(imm)],
                })
            }
            0b0000011 => {
                let (data_width, is_unsigned) = match func3 {
                    0b000 => (DataWidth::Byte, false),
                    0b001 => (DataWidth::HalfWord, false),
                    0b010 => (DataWidth::Word, false),
                    0b100 => (DataWidth::Byte, true),
                    0b101 => (DataWidth::HalfWord, true),
                    _ => return Err(Exception::IllegalInstruction),
                };
                Ok(Instruction {
                    opcode: Opcode::Load(data_width, is_unsigned),
                    operand: [Operand::Reg(rd), Operand::Reg(rs1), Operand::Imm(imm)],
                })
            }
            0b0100011 => {
                let mut imm: BitVec<_, Lsb0> = BitVec::from_element(0u32);
                imm[0..=4].store(inst.get(7..=11).unwrap().load::<u32>());
                imm[5..=11].store(inst.get(25..=31).unwrap().load::<u32>());
                let imm = signed_extend(imm.load::<u32>(), 11 + 1);

                Ok(Instruction {
                    opcode: Opcode::Store(match func3 {
                        0b000 => DataWidth::Byte,
                        0b001 => DataWidth::HalfWord,
                        0b010 => DataWidth::Word,
                        _ => return Err(Exception::IllegalInstruction),
                    }),
                    operand: [Operand::Reg(rs1), Operand::Reg(rs2), Operand::Imm(imm)],
                })
            }
            0b0010011 => {
                let opcode = match func3 {
                    0b000 => Opcode::Add,
                    0b001 => Opcode::Sll,
                    0b010 => Opcode::Slt(false),
                    0b011 => Opcode::Slt(true),
                    0b100 => Opcode::Xor,
                    0b101 => match func7 {
                        0b0000000 => Opcode::Srl,
                        0b0100000 => Opcode::Sra,
                        _ => return Err(Exception::IllegalInstruction),
                    },
                    0b110 => Opcode::Or,
                    0b111 => Opcode::And,
                    _ => return Err(Exception::IllegalInstruction),
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
                    operand: [Operand::Reg(rd), Operand::Reg(rs1), operand],
                })
            }
            0b0110011 => match func7 {
                0b0000001 => Ok(Instruction {
                    opcode: match func3 {
                        0b000 => Opcode::Mul(false, false, false),
                        0b001 => Opcode::Mul(true, false, false),
                        0b010 => Opcode::Mul(true, false, true),
                        0b011 => Opcode::Mul(true, true, true),
                        0b100 => Opcode::Div(false, false),
                        0b101 => Opcode::Div(false, true),
                        0b110 => Opcode::Div(true, false),
                        0b111 => Opcode::Div(true, true),
                        _ => return Err(Exception::IllegalInstruction),
                    },
                    operand: [Operand::Reg(rd), Operand::Reg(rs1), Operand::Reg(rs2)],
                }),
                _ => Ok(Instruction {
                    opcode: match func3 {
                        0b000 => match func7 {
                            0b0000000 => Opcode::Add,
                            0b0100000 => Opcode::Sub,
                            _ => return Err(Exception::IllegalInstruction),
                        },
                        0b001 => Opcode::Sll,
                        0b010 => Opcode::Slt(false),
                        0b011 => Opcode::Slt(true),
                        0b100 => Opcode::Xor,
                        0b101 => match func7 {
                            0b0000000 => Opcode::Srl,
                            0b0100000 => Opcode::Sra,
                            _ => return Err(Exception::IllegalInstruction),
                        },
                        0b110 => Opcode::Or,
                        0b111 => Opcode::And,
                        _ => return Err(Exception::IllegalInstruction),
                    },
                    operand: [Operand::Reg(rd), Operand::Reg(rs1), Operand::Reg(rs2)],
                }),
            },
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
                0b0011000 => Ok(Instruction {
                    opcode: Opcode::MRet,
                    operand: [Operand::Nothing, Operand::Nothing, Operand::Nothing],
                }),
                _ => {
                    let csr = inst.get(20..=31).unwrap().load();
                    let (opcode, operand) = match func3 {
                        0b001 => (Opcode::CSRWrite, Operand::Reg(rs1)),
                        0b010 => (Opcode::CSRSet, Operand::Reg(rs1)),
                        0b011 => (Opcode::CSRClear, Operand::Reg(rs1)),
                        0b101 => (Opcode::CSRWrite, Operand::Imm(rs1 as u32)),
                        0b110 => (Opcode::CSRSet, Operand::Imm(rs1 as u32)),
                        0b111 => (Opcode::CSRClear, Operand::Imm(rs1 as u32)),
                        _ => unreachable!(),
                    };
                    Ok(Instruction {
                        opcode,
                        operand: [Operand::Reg(rd), Operand::Csr(csr), operand],
                    })
                }
            },
            _ => Err(Exception::IllegalInstruction),
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
