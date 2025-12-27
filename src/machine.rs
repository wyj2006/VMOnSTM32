use yaxpeax_arch::{Decoder, ReadError, Reader};
use yaxpeax_arm::armv7::{ConditionCode, InstDecoder, Operand, RegShiftStyle};

use crate::arithmetic::*;
use crate::cpu::{CPU, InstrSet, PC_INDEX};
use crate::memory::Memory;
use crate::vmerror::VMError;

pub struct Machine {
    pub cpu: CPU,
    pub arch_version: u32,
    pub memory: Memory,
    pub mark: u32,
}

impl Default for Machine {
    fn default() -> Self {
        let cpu = CPU::default();
        Machine {
            arch_version: 7,
            memory: Memory::default(),
            mark: cpu.regs[PC_INDEX],
            cpu,
        }
    }
}

impl Reader<u32, u8> for Machine {
    fn next(&mut self) -> Result<u8, ReadError> {
        let address = self.cpu.regs[PC_INDEX];
        if (address as usize) >= self.memory.size() {
            return Err(ReadError::ExhaustedInput);
        }
        self.cpu.regs[PC_INDEX] += 1;
        Ok(self.read_memory(address)?)
    }

    fn next_n(&mut self, buf: &mut [u8]) -> Result<(), ReadError> {
        if buf.len() + self.cpu.regs[PC_INDEX] as usize > self.memory.size() {
            return Err(ReadError::ExhaustedInput);
        }
        for i in 0..buf.len() {
            buf[i] = self.next()?;
        }
        Ok(())
    }

    fn mark(&mut self) {
        self.mark = self.cpu.regs[PC_INDEX]
    }

    fn offset(&mut self) -> u32 {
        self.cpu.regs[PC_INDEX] - self.mark
    }

    fn total_offset(&mut self) -> u32 {
        self.cpu.regs[PC_INDEX]
    }
}

impl Machine {
    /* P2639
    IsZero(x) = (BitCount(x) == 0)
    IsOnes(x) = (BitCount(x) == Len(x))
    IsZeroBit(x) = if IsZero(x) then '1' else '0'
    IsOnesBit(x) = if IsOnes(x) then '1' else '0'
    */
    //P287
    pub fn condition_passed(&self, cond: ConditionCode) -> bool {
        let cond = cond as u8;
        let apsr = self.cpu.apsr();
        let mut result = match cond >> 1 & 0b111 {
            0b000 => apsr.z(),                                  // EQ or NE
            0b001 => apsr.c(),                                  // CS or CC
            0b010 => apsr.n(),                                  // MI or PL
            0b011 => apsr.v(),                                  // VS or VC
            0b100 => apsr.c() && apsr.z() == false,             // HI or LS
            0b101 => apsr.n() == apsr.v(),                      // GE or LT
            0b110 => apsr.n() == apsr.v() && apsr.z() == false, // GT or LE
            0b111 => true,                                      // AL
            _ => unreachable!(),
        };
        if cond & 1 == 1 && cond != 0b1111 {
            result = !result;
        }
        result
    }

    //P48
    pub fn alu_write_pc(&mut self, address: u32) {
        if self.arch_version >= 7
            && let InstrSet::Arm = self.current_instr_set()
        {
            self.bw_write_pc(address);
        } else {
            self.branch_write_pc(address);
        }
    }

    //P47
    //跳转但不切换指令集
    pub fn branch_write_pc(&mut self, address: u32) {
        match self.current_instr_set() {
            InstrSet::Arm => self.branch_to(address & !0b11),
            InstrSet::Jazelle => unimplemented!(),
            _ => self.branch_to(address & !0b1),
        }
    }

    //P47
    //跳转但可以切换指令集
    pub fn bw_write_pc(&mut self, address: u32) {
        match self.current_instr_set() {
            InstrSet::ThumbEE => unimplemented!(),
            _ => {
                if address & 1 == 1 {
                    self.select_instr_set(InstrSet::Thumb);
                    self.branch_to(address & !0b1);
                } else if address >> 1 & 1 == 0 {
                    self.select_instr_set(InstrSet::Arm);
                    self.branch_to(address);
                }
            }
        }
    }

    //P47
    pub fn load_write_pc(&mut self, address: u32) {
        if self.arch_version >= 5 {
            self.bw_write_pc(address);
        } else {
            self.branch_write_pc(address);
        }
    }

    //1147
    pub fn branch_to(&mut self, address: u32) {
        self.cpu.regs[PC_INDEX] = address;
    }

    // P2641
    pub fn align(&self, address: u32, alignment: u32) -> u32 {
        (address + alignment - 1) & !(alignment - 1)
    }

    pub fn read_address(&self, operand: Operand) -> Result<u32, VMError> {
        Ok(match operand {
            Operand::RegDeref(reg) => self.cpu.regs[reg.number() as usize],
            Operand::RegDerefPostindexOffset(reg, ..) => self.cpu.regs[reg.number() as usize],
            Operand::RegDerefPostindexReg(reg, ..) => self.cpu.regs[reg.number() as usize],
            Operand::RegDerefPostindexRegShift(reg, ..) => self.cpu.regs[reg.number() as usize],
            Operand::RegDerefPreindexOffset(reg, offset, add, ..) => {
                let a = self.cpu.regs[reg.number() as usize];
                let b = offset as u32;
                if add { a + b } else { a - b }
            }
            Operand::RegDerefPreindexReg(reg, reg2, add, ..) => {
                let a = self.cpu.regs[reg.number() as usize];
                let b = self.cpu.regs[reg2.number() as usize];
                if add { a + b } else { a - b }
            }
            Operand::RegDerefPreindexRegShift(reg, reg_shift, add, ..) => {
                let a = self.cpu.regs[reg.number() as usize];
                let b = self.read(Operand::RegShift(reg_shift))?;
                if add { a + b } else { a - b }
            }
            _ => unreachable!(),
        })
    }

    pub fn read(&self, operand: Operand) -> Result<u32, VMError> {
        Ok(match operand {
            Operand::Imm32(value) => value,
            Operand::Imm12(value) => value as u32,
            Operand::Reg(reg) => self.cpu.regs[reg.number() as usize],
            Operand::RegShift(reg_shift) => {
                let reg;
                let shift_style;
                let amount;
                match reg_shift.into_shift() {
                    RegShiftStyle::RegImm(reg_imm_shift) => {
                        shift_style = reg_imm_shift.stype();
                        amount = reg_imm_shift.imm() as u32;
                        reg = reg_imm_shift.shiftee();
                    }
                    RegShiftStyle::RegReg(reg_reg_shift) => {
                        shift_style = reg_reg_shift.stype();
                        reg = reg_reg_shift.shiftee();
                        amount = self.cpu.regs[reg_reg_shift.shifter().number() as usize];
                    }
                }
                shift(
                    self.cpu.regs[reg.number() as usize],
                    shift_style,
                    amount,
                    self.cpu.apsr().c(),
                )
            }
            // u32 as i32和i32 as u32都只改变解释方式
            Operand::BranchOffset(value) => (value << 2) as u32,
            Operand::BranchThumbOffset(value) => (value << 1) as u32,
            Operand::RegWBack(reg, _wback) => self.cpu.regs[reg.number() as usize],
            Operand::RegList(registers) => registers as u32,
            Operand::RegDeref(..)
            | Operand::RegDerefPostindexOffset(..)
            | Operand::RegDerefPostindexReg(..)
            | Operand::RegDerefPostindexRegShift(..)
            | Operand::RegDerefPreindexOffset(..)
            | Operand::RegDerefPreindexReg(..)
            | Operand::RegDerefPreindexRegShift(..) => {
                self.read_memory_word(self.read_address(operand)?)?
            }
            Operand::APSR => self.cpu.apsr().0,
            Operand::CPSR => self.cpu.cpsr.0,
            Operand::SPSR => unimplemented!(), //TODO Operand::SPSR
            _ => unimplemented!(),
        })
    }

    pub fn write(&mut self, operand: Operand, value: u32) -> Result<(), VMError> {
        match operand {
            Operand::Reg(reg) => self.cpu.regs[reg.number() as usize] = value,
            Operand::RegWBack(reg, true) => self.cpu.regs[reg.number() as usize] = value,
            Operand::RegDerefPostindexOffset(reg, offset, add, true) => {
                let reg = Operand::Reg(reg);
                let b = offset as u32;
                if add {
                    self.write(reg, value + b)?;
                } else {
                    self.write(reg, value - b)?;
                }
            }
            Operand::RegDerefPostindexReg(reg, reg2, add, true) => {
                let reg = Operand::Reg(reg);
                let b = self.cpu.regs[reg2.number() as usize];
                if add {
                    self.write(reg, value + b)?;
                } else {
                    self.write(reg, value - b)?;
                }
            }
            Operand::RegDerefPostindexRegShift(reg, reg_shift, add, true) => {
                let reg = Operand::Reg(reg);
                let b = self.read(Operand::RegShift(reg_shift))?;
                if add {
                    self.write(reg, value + b)?;
                } else {
                    self.write(reg, value - b)?;
                }
            }
            Operand::RegDerefPreindexOffset(reg, .., true) => {
                self.write(Operand::Reg(reg), value)?
            }
            Operand::RegDerefPreindexReg(reg, .., true) => self.write(Operand::Reg(reg), value)?,
            Operand::RegDerefPreindexRegShift(reg, .., true) => {
                self.write(Operand::Reg(reg), value)?
            }
            Operand::StatusRegMask(..) => {} //TODO StatusRegMask
            _ => {}
        }
        Ok(())
    }

    pub fn run(&mut self) -> ! {
        loop {
            let mut decoder = InstDecoder::armv7();
            decoder.set_thumb_mode(if let InstrSet::Thumb = self.current_instr_set() {
                true
            } else {
                false
            });
            let instruction = match decoder.decode(self) {
                Ok(t) => t,
                Err(_) => todo!(), //TODO 处理非法的指令
            };
            self.execute(instruction).unwrap();
        }
    }
}
