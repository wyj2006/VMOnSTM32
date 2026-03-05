use crate::{
    cpu::Mode,
    decode::{Condition, DataWidth, Instruction, Opcode, Operand},
    exception::Exception,
    machine::Machine,
};

impl Machine {
    pub fn read(&mut self, operand: Operand) -> Result<u32, Exception> {
        Ok(match operand {
            Operand::Imm(t) => t,
            Operand::Reg(t) => self.cpu.regs[t],
            Operand::Csr(address) => *self.cpu.get_csr_mut(address, false)?,
            Operand::Nothing => 0,
        })
    }

    pub fn write(&mut self, operand: Operand, value: u32) -> Result<(), Exception> {
        match operand {
            Operand::Reg(t) => self.cpu.regs[t] = value,
            Operand::Csr(address) => *self.cpu.get_csr_mut(address, true)? = value,
            Operand::Imm(..) | Operand::Nothing => {}
        }
        Ok(())
    }

    pub fn execute(&mut self, inst: Instruction) -> Result<(), Exception> {
        match inst.opcode {
            Opcode::Lui => {
                let rd = inst.operand[0];
                let imm = self.read(inst.operand[1])?;
                self.write(rd, imm)?;
            }
            Opcode::Auipc => {
                let rd = inst.operand[0];
                let imm = self.read(inst.operand[1])?;
                self.write(rd, self.cpu.pc + imm)?;
            }
            Opcode::Jal => {
                let rd = inst.operand[0];
                let imm = self.read(inst.operand[1])?;
                self.write(rd, self.cpu.pc)?;
                self.cpu.pc += imm - 4;
            }
            Opcode::Jalr => {
                let rd = inst.operand[0];
                let rs1 = self.read(inst.operand[1])?;
                let imm = self.read(inst.operand[2])?;
                self.write(rd, self.cpu.pc)?;
                self.cpu.pc = (rs1 + imm) & !1u32;
            }
            Opcode::Branch(cond) => {
                let rs1 = self.read(inst.operand[0])? as i32;
                let rs2 = self.read(inst.operand[1])? as i32;
                let imm = self.read(inst.operand[2])?;
                if match cond {
                    Condition::Eq => rs1 == rs2,
                    Condition::Neq => rs1 != rs2,
                    Condition::Lt => rs1 < rs2,
                    Condition::Ge => rs1 > rs2,
                    Condition::Ltu => (rs1 as u32) < (rs2 as u32),
                    Condition::Geu => (rs1 as u32) > (rs2 as u32),
                } {
                    self.cpu.pc += imm - 4;
                }
            }
            Opcode::Load(data_width, is_unsigned) => {
                let rd = inst.operand[0];
                let rs1 = self.read(inst.operand[1])?;
                let imm = self.read(inst.operand[2])?;

                let address = (rs1 + imm) as usize;
                self.write(
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
            Opcode::Store(data_width) => {
                let rs1 = self.read(inst.operand[0])?;
                let rs2 = self.read(inst.operand[1])?;
                let imm = self.read(inst.operand[2])?;

                let address = (rs1 + imm) as usize;
                match data_width {
                    DataWidth::Byte => self.memory.write(address, rs2 as u8)?,
                    DataWidth::HalfWord => self.memory.write(address, rs2 as u16)?,
                    DataWidth::Word => self.memory.write(address, rs2 as u32)?,
                }
            }
            Opcode::Add | Opcode::Sub | Opcode::Xor | Opcode::Or | Opcode::And => {
                let rd = inst.operand[0];
                let a = self.read(inst.operand[1])?;
                let b = self.read(inst.operand[2])?;
                self.write(
                    rd,
                    match inst.opcode {
                        Opcode::Add => a + b,
                        Opcode::Sub => a - b,
                        Opcode::Xor => a ^ b,
                        Opcode::Or => a | b,
                        Opcode::And => a & b,
                        _ => unreachable!(),
                    },
                )?;
            }
            Opcode::Slt(is_unsigned) => {
                let rd = inst.operand[0];
                let a = self.read(inst.operand[1])?;
                let b = self.read(inst.operand[2])?;
                self.write(
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
            Opcode::Sll | Opcode::Srl | Opcode::Sra => {
                let rd = inst.operand[0];
                let rs1 = self.read(inst.operand[1])?;
                let shamt = self.read(inst.operand[2])?;
                self.write(
                    rd,
                    match inst.opcode {
                        Opcode::Sll => rs1 << shamt,
                        Opcode::Srl => rs1 >> shamt,
                        Opcode::Sra => (rs1 as i32 >> shamt) as u32,
                        _ => unreachable!(),
                    },
                )?;
            }
            Opcode::Mul(keep_high, a_is_unsigned, b_is_unsigned) => {
                let rd = inst.operand[0];
                let a = self.read(inst.operand[1])?;
                let b = self.read(inst.operand[2])?;
                let value = match (a_is_unsigned, b_is_unsigned) {
                    (true, true) => a as u64 * b as u64,
                    (true, false) => (a as u64 as i64 * b as i32 as i64) as u64,
                    (false, true) => (a as i32 as i64 * b as u64 as i64) as u64,
                    (false, false) => (a as i32 as i64 * b as i32 as i64) as u64,
                };

                self.write(
                    rd,
                    if keep_high {
                        value >> 32
                    } else {
                        value & 0xffffffff
                    } as u32,
                )?;
            }
            Opcode::Div(keep_rem, is_unsigned) => {
                let rd = inst.operand[0];
                let a = self.read(inst.operand[1])?;
                let b = self.read(inst.operand[2])?;
                self.write(
                    rd,
                    match (keep_rem, is_unsigned) {
                        (false, false) => (a as i32 / b as i32) as u32,
                        (false, true) => a / b,
                        (true, false) => (a as i32 % b as i32) as u32,
                        (true, true) => a % b,
                    },
                )?;
            }
            Opcode::CSRWrite | Opcode::CSRClear | Opcode::CSRSet => {
                let rd = inst.operand[0];
                let csr = inst.operand[1];
                let a = self.read(inst.operand[2])?;

                let old = self.read(csr)?;
                self.write(rd, old)?;

                self.write(
                    csr,
                    match inst.opcode {
                        Opcode::CSRWrite => a,
                        Opcode::CSRSet => old | a,
                        Opcode::CSRClear => old & !a,
                        _ => unreachable!(),
                    },
                )?;
            }
            Opcode::Wfi => {
                self.check_mode(Mode::Machine)?;
            }
            Opcode::SRet => {
                self.check_mode(Mode::Machine)?;
                todo!();
            }
            Opcode::MRet => {
                self.check_mode(Mode::Machine)?;
                self.cpu.pc = self.cpu.mepc;
                self.cpu.mstatus.set_mie(self.cpu.mstatus.mpie());
                self.cpu.mode = Mode::from(self.cpu.mstatus.mpp());
            }
        }
        Ok(())
    }

    pub fn check_mode(&self, at_least: Mode) -> Result<(), Exception> {
        if (self.cpu.mode as u32) < (at_least as u32) {
            Err(Exception::IllegalInstruction)
        } else {
            Ok(())
        }
    }
}
