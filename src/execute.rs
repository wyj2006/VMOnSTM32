use crate::{
    cpu::Mode,
    decode::{Condition, DataType, DataWidth, Instruction, Opcode, Operand, SignJoinKind},
    machine::Machine,
    vm_exception::VMException,
};
use core::num::FpCategory;
use libm::{sqrt, sqrtf};

impl Machine {
    pub fn readi(&mut self, operand: Operand) -> Result<u32, VMException> {
        Ok(match operand {
            Operand::Imm(t) => t,
            Operand::XReg(t) => self.cpu.xregs[t],
            Operand::Csr(address) => *self.cpu.get_csr_mut(address, false)?,
            _ => unreachable!(),
        })
    }

    pub fn readf(&mut self, operand: Operand) -> Result<f32, VMException> {
        Ok(match operand {
            Operand::FReg(t) => f32::from_bits((self.cpu.dregs[t].to_bits() & 0xffffffff) as u32),
            _ => self.readi(operand)? as f32,
        })
    }

    pub fn readd(&mut self, operand: Operand) -> Result<f64, VMException> {
        Ok(match operand {
            Operand::DReg(t) => self.cpu.dregs[t],
            _ => self.readf(operand)? as f64,
        })
    }

    pub fn writei(&mut self, operand: Operand, value: u32) -> Result<(), VMException> {
        match operand {
            Operand::XReg(t) => self.cpu.xregs[t] = value,
            Operand::Csr(address) => *self.cpu.get_csr_mut(address, true)? = value,
            _ => unreachable!(),
        }
        Ok(())
    }

    pub fn writef(&mut self, operand: Operand, value: f32) -> Result<(), VMException> {
        match operand {
            Operand::FReg(t) => {
                self.cpu.dregs[t] = f64::from_bits(
                    (self.cpu.dregs[t].to_bits() & !0xffffffff) | value.to_bits() as u64,
                )
            }
            _ => self.writei(operand, value as u32)?,
        }
        Ok(())
    }

    pub fn writed(&mut self, operand: Operand, value: f64) -> Result<(), VMException> {
        match operand {
            Operand::DReg(t) => self.cpu.dregs[t] = value,
            _ => self.writef(operand, value as f32)?,
        }
        Ok(())
    }

    pub fn execute(&mut self, inst: Instruction) -> Result<(), VMException> {
        match inst.opcode {
            Opcode::Auipc => {
                let rd = inst.operand[0];
                let imm = self.readi(inst.operand[1])?;
                self.writei(rd, self.cpu.pc + imm)?;
            }
            Opcode::Branch(cond) => {
                let rs1 = self.readi(inst.operand[0])? as i32;
                let rs2 = self.readi(inst.operand[1])? as i32;
                let imm = self.readi(inst.operand[2])?;
                if match cond {
                    Condition::Eq => rs1 == rs2,
                    Condition::Neq => rs1 != rs2,
                    Condition::Lt => rs1 < rs2,
                    Condition::Ge => rs1 > rs2,
                    Condition::Ltu => (rs1 as u32) < (rs2 as u32),
                    Condition::Geu => (rs1 as u32) > (rs2 as u32),
                    _ => unreachable!(),
                } {
                    self.cpu.pc += imm - 4;
                }
            }
            Opcode::CSRWrite | Opcode::CSRClear | Opcode::CSRSet => {
                let rd = inst.operand[0];
                let csr = inst.operand[1];
                let a = self.readi(inst.operand[2])?;

                let old = self.readi(csr)?;
                self.writei(rd, old)?;

                self.writei(
                    csr,
                    match inst.opcode {
                        Opcode::CSRWrite => a,
                        Opcode::CSRSet => old | a,
                        Opcode::CSRClear => old & !a,
                        _ => unreachable!(),
                    },
                )?;
            }
            Opcode::FAdd(data_type)
            | Opcode::FSub(data_type)
            | Opcode::FMul(data_type)
            | Opcode::FDiv(data_type) => {
                let rd = inst.operand[0];
                match data_type {
                    DataType::Double => {
                        let a = self.readd(inst.operand[1])?;
                        let b = self.readd(inst.operand[2])?;
                        self.writed(
                            rd,
                            match inst.opcode {
                                Opcode::FAdd(..) => a + b,
                                Opcode::FSub(..) => a - b,
                                Opcode::FMul(..) => a * b,
                                Opcode::FDiv(..) => a / b,
                                _ => unreachable!(),
                            },
                        )?;
                    }
                    DataType::Float => {
                        let a = self.readf(inst.operand[1])?;
                        let b = self.readf(inst.operand[2])?;
                        self.writef(
                            rd,
                            match inst.opcode {
                                Opcode::FAdd(..) => a + b,
                                Opcode::FSub(..) => a - b,
                                Opcode::FMul(..) => a * b,
                                Opcode::FDiv(..) => a / b,
                                _ => unreachable!(),
                            },
                        )?;
                    }
                    _ => unreachable!(),
                }
            }
            Opcode::FClassify(data_type) => {
                let rd = inst.operand[0];
                let (positive, classify) = match data_type {
                    DataType::Double => {
                        let value = self.readd(inst.operand[1])?;
                        (value.is_sign_positive(), value.classify())
                    }
                    _ => unreachable!(),
                };
                self.writei(
                    rd,
                    match classify {
                        FpCategory::Infinite if !positive => 0,
                        FpCategory::Normal if !positive => 1,
                        FpCategory::Subnormal if !positive => 2,
                        FpCategory::Zero if !positive => 3,
                        FpCategory::Zero if positive => 4,
                        FpCategory::Subnormal if positive => 5,
                        FpCategory::Normal if !positive => 6,
                        FpCategory::Infinite if positive => 7,
                        FpCategory::Nan => todo!(),
                        _ => unreachable!(),
                    },
                )?;
            }
            Opcode::FCompare(cond, data_type) => {
                let rd = inst.operand[0];
                let value = match data_type {
                    DataType::Double => {
                        let a = self.readd(inst.operand[1])?;
                        let b = self.readd(inst.operand[2])?;
                        (match cond {
                            Condition::Eq => a == b,
                            Condition::Lt => a < b,
                            Condition::Le => a <= b,
                            _ => unreachable!(),
                        } as u32)
                    }
                    DataType::Float => {
                        let a = self.readf(inst.operand[1])?;
                        let b = self.readf(inst.operand[2])?;
                        (match cond {
                            Condition::Eq => a == b,
                            Condition::Lt => a < b,
                            Condition::Le => a <= b,
                            _ => unreachable!(),
                        } as u32)
                    }
                    _ => unreachable!(),
                };
                self.writei(rd, value)?;
            }
            Opcode::FConvert(from, to) => {
                let rd = inst.operand[0];
                match from {
                    DataType::Double => {
                        let value = self.readd(inst.operand[1])?;
                        match to {
                            DataType::Double => self.writed(rd, value as f64)?,
                            DataType::Float => self.writef(rd, value as f32)?,
                            DataType::I32 => self.writei(rd, value as u32)?,
                        }
                    }
                    DataType::Float => {
                        let value = self.readf(inst.operand[1])?;
                        match to {
                            DataType::Double => self.writed(rd, value as f64)?,
                            DataType::Float => self.writef(rd, value as f32)?,
                            DataType::I32 => self.writei(rd, value as u32)?,
                        }
                    }
                    DataType::I32 => {
                        let value = self.readi(inst.operand[1])?;
                        match to {
                            DataType::Double => self.writed(rd, value as f64)?,
                            DataType::Float => self.writef(rd, value as f32)?,
                            DataType::I32 => self.writei(rd, value as u32)?,
                        }
                    }
                }
            }
            Opcode::FLoad(data_type) => {
                let rd = inst.operand[0];
                let rs1 = self.readi(inst.operand[1])?;
                let imm = self.readi(inst.operand[2])?;

                let address = (rs1 + imm) as usize;
                match data_type {
                    DataType::Double => self.writed(
                        rd,
                        f64::from_le_bytes(self.memory.read::<u64>(address)?.to_le_bytes()),
                    )?,
                    DataType::Float => self.writef(
                        rd,
                        f32::from_le_bytes(self.memory.read::<u32>(address)?.to_le_bytes()),
                    )?,
                    _ => unreachable!(),
                }
            }
            Opcode::FMax(data_type) | Opcode::FMin(data_type) => {
                let rd = inst.operand[0];
                match data_type {
                    DataType::Double => {
                        let a = self.readd(inst.operand[1])?;
                        let b = self.readd(inst.operand[2])?;
                        self.writed(
                            rd,
                            match inst.opcode {
                                Opcode::FMax(..) => a.max(b),
                                Opcode::FMin(..) => a.min(b),
                                _ => unreachable!(),
                            },
                        )?;
                    }
                    DataType::Float => {
                        let a = self.readf(inst.operand[1])?;
                        let b = self.readf(inst.operand[2])?;
                        self.writef(
                            rd,
                            match inst.opcode {
                                Opcode::FMax(..) => a.max(b),
                                Opcode::FMin(..) => a.min(b),
                                _ => unreachable!(),
                            },
                        )?;
                    }
                    _ => unreachable!(),
                }
            }
            Opcode::FMove(from, to) => {
                let rd = inst.operand[0];
                let bits = match from {
                    DataType::Double => self.readd(inst.operand[1])?.to_bits(),
                    DataType::Float => self.readf(inst.operand[1])?.to_bits() as u64,
                    DataType::I32 => self.readi(inst.operand[1])? as u64,
                };
                match to {
                    DataType::Double => self.writed(rd, f64::from_bits(bits))?,
                    DataType::Float => self.writef(rd, f32::from_bits(bits as u32))?,
                    DataType::I32 => self.writei(rd, bits as u32)?,
                }
            }
            Opcode::FSignJoin(kind, data_type) => {
                let rd = inst.operand[0];
                match data_type {
                    DataType::Double => {
                        let a = self.readd(inst.operand[1])?;
                        let b = self.readd(inst.operand[2])?;
                        let a_sign = if a.signum() == -1. { 0 } else { 1 };
                        let b_sign = if b.signum() == -1. { 0 } else { 1 };
                        self.writed(
                            rd,
                            a.abs()
                                * (match kind {
                                    SignJoinKind::Default => b_sign,
                                    SignJoinKind::Negative => -b_sign,
                                    SignJoinKind::Xor => a_sign ^ b_sign,
                                } * 2
                                    - 1) as f64,
                        )?;
                    }
                    DataType::Float => {
                        let a = self.readf(inst.operand[1])?;
                        let b = self.readf(inst.operand[2])?;
                        let a_sign = if a.signum() == -1. { 0 } else { 1 };
                        let b_sign = if b.signum() == -1. { 0 } else { 1 };
                        self.writef(
                            rd,
                            a.abs()
                                * (match kind {
                                    SignJoinKind::Default => b_sign,
                                    SignJoinKind::Negative => -b_sign,
                                    SignJoinKind::Xor => a_sign ^ b_sign,
                                } * 2
                                    - 1) as f32,
                        )?;
                    }
                    _ => unreachable!(),
                }
            }
            Opcode::FSqrt(data_type) => {
                let rd = inst.operand[0];
                match data_type {
                    DataType::Double => {
                        let a = self.readd(inst.operand[1])?;
                        self.writed(rd, sqrt(a))?;
                    }
                    DataType::Float => {
                        let a = self.readf(inst.operand[1])?;
                        self.writef(rd, sqrtf(a))?;
                    }
                    _ => unreachable!(),
                }
            }
            Opcode::FStore(data_type) => {
                let rs1 = self.readi(inst.operand[0])?;
                let imm = self.readi(inst.operand[2])?;

                let address = (rs1 + imm) as usize;
                match data_type {
                    DataType::Double => {
                        let rs2 = self.readd(inst.operand[1])?;
                        let bits = rs2.to_bits();
                        self.memory.write(address, (bits & 0xffffffff) as u32)?;
                        self.memory.write(address + 4, (bits >> 32) as u32)?;
                    }
                    DataType::Float => {
                        let rs2 = self.readf(inst.operand[1])?;
                        self.memory
                            .write(address, u32::from_le_bytes(rs2.to_le_bytes()))?;
                    }
                    _ => unreachable!(),
                }
            }
            Opcode::IAdd | Opcode::ISub | Opcode::Xor | Opcode::Or | Opcode::IAnd => {
                let rd = inst.operand[0];
                let a = self.readi(inst.operand[1])?;
                let b = self.readi(inst.operand[2])?;
                self.writei(
                    rd,
                    match inst.opcode {
                        Opcode::IAdd => a + b,
                        Opcode::ISub => a - b,
                        Opcode::Xor => a ^ b,
                        Opcode::Or => a | b,
                        Opcode::IAnd => a & b,
                        _ => unreachable!(),
                    },
                )?;
            }
            Opcode::IDiv(keep_rem, is_unsigned) => {
                let rd = inst.operand[0];
                let a = self.readi(inst.operand[1])?;
                let b = self.readi(inst.operand[2])?;
                self.writei(
                    rd,
                    match (keep_rem, is_unsigned) {
                        (false, false) => (a as i32 / b as i32) as u32,
                        (false, true) => a / b,
                        (true, false) => (a as i32 % b as i32) as u32,
                        (true, true) => a % b,
                    },
                )?;
            }
            Opcode::ILoad(data_width, is_unsigned) => {
                let rd = inst.operand[0];
                let rs1 = self.readi(inst.operand[1])?;
                let imm = self.readi(inst.operand[2])?;

                let address = (rs1 + imm) as usize;
                self.writei(
                    rd,
                    match data_width {
                        DataWidth::Byte if is_unsigned => self.memory.read::<u8>(address)? as u32,
                        DataWidth::Byte if !is_unsigned => {
                            self.memory.read::<i8>(address)? as i32 as u32
                        }
                        DataWidth::HalfWord if is_unsigned => {
                            self.memory.read::<u16>(address)? as u32
                        }
                        DataWidth::HalfWord if !is_unsigned => {
                            self.memory.read::<i16>(address)? as i32 as u32
                        }
                        DataWidth::Word if is_unsigned => self.memory.read::<u32>(address)?,
                        DataWidth::Word if !is_unsigned => self.memory.read::<i32>(address)? as u32,
                        _ => unreachable!(),
                    },
                )?;
            }
            Opcode::IMul(keep_high, a_is_unsigned, b_is_unsigned) => {
                let rd = inst.operand[0];
                let a = self.readi(inst.operand[1])?;
                let b = self.readi(inst.operand[2])?;
                let value = match (a_is_unsigned, b_is_unsigned) {
                    (true, true) => a as u64 * b as u64,
                    (true, false) => (a as u64 as i64 * b as i32 as i64) as u64,
                    (false, true) => (a as i32 as i64 * b as u64 as i64) as u64,
                    (false, false) => (a as i32 as i64 * b as i32 as i64) as u64,
                };

                self.writei(
                    rd,
                    if keep_high {
                        value >> 32
                    } else {
                        value & 0xffffffff
                    } as u32,
                )?;
            }
            Opcode::IStore(data_width) => {
                let rs1 = self.readi(inst.operand[0])?;
                let rs2 = self.readi(inst.operand[1])?;
                let imm = self.readi(inst.operand[2])?;

                let address = (rs1 + imm) as usize;
                match data_width {
                    DataWidth::Byte => self.memory.write(address, rs2 as u8)?,
                    DataWidth::HalfWord => self.memory.write(address, rs2 as u16)?,
                    DataWidth::Word => self.memory.write(address, rs2 as u32)?,
                }
            }
            Opcode::Jal => {
                let rd = inst.operand[0];
                let imm = self.readi(inst.operand[1])?;
                self.writei(rd, self.cpu.pc)?;
                self.cpu.pc += imm - 4;
            }
            Opcode::Jalr => {
                let rd = inst.operand[0];
                let rs1 = self.readi(inst.operand[1])?;
                let imm = self.readi(inst.operand[2])?;
                self.writei(rd, self.cpu.pc)?;
                self.cpu.pc = (rs1 + imm) & !1u32;
            }
            Opcode::Lui => {
                let rd = inst.operand[0];
                let imm = self.readi(inst.operand[1])?;
                self.writei(rd, imm)?;
            }
            Opcode::MRet => {
                self.check_mode(Mode::Machine)?;
                self.cpu.pc = self.cpu.mepc;
                self.cpu.mstatus.set_mie(self.cpu.mstatus.mpie());
                self.cpu.mode = Mode::from(self.cpu.mstatus.mpp());
            }

            Opcode::Sll | Opcode::Srl | Opcode::Sra => {
                let rd = inst.operand[0];
                let rs1 = self.readi(inst.operand[1])?;
                let shamt = self.readi(inst.operand[2])?;
                self.writei(
                    rd,
                    match inst.opcode {
                        Opcode::Sll => rs1 << shamt,
                        Opcode::Srl => rs1 >> shamt,
                        Opcode::Sra => (rs1 as i32 >> shamt) as u32,
                        _ => unreachable!(),
                    },
                )?;
            }
            Opcode::Slt(is_unsigned) => {
                let rd = inst.operand[0];
                let a = self.readi(inst.operand[1])?;
                let b = self.readi(inst.operand[2])?;
                self.writei(
                    rd,
                    if match is_unsigned {
                        true => a < b,
                        false => (a as i32) < (b as i32),
                    } {
                        1
                    } else {
                        0
                    },
                )?;
            }
            Opcode::SRet => {
                self.check_mode(Mode::Machine)?;
                todo!();
            }
            Opcode::Wfi => {
                self.check_mode(Mode::Machine)?;
            }
        }
        Ok(())
    }

    pub fn check_mode(&self, at_least: Mode) -> Result<(), VMException> {
        if (self.cpu.mode as u32) < (at_least as u32) {
            Err(VMException::IllegalInstruction)
        } else {
            Ok(())
        }
    }
}
