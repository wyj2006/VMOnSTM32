use yaxpeax_arch::{Decoder, ReadError, Reader};
use yaxpeax_arm::armv7::{
    ConditionCode, InstDecoder, Instruction, Opcode, Operand, RegShiftStyle, ShiftStyle,
};

use crate::arithmetic::*;
use crate::cpu::{CPU, InstrSet, LR_INDEX, PC_INDEX};
use crate::memory::Memory;

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
        Ok(self.read_memory(address))
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

    //1147
    pub fn branch_to(&mut self, address: u32) {
        self.cpu.regs[PC_INDEX] = address;
    }

    // P2641
    pub fn align(&self, address: u32, alignment: u32) -> u32 {
        (address + alignment - 1) & !(alignment - 1)
    }

    pub fn read(&self, operand: Operand) -> u32 {
        match operand {
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
            Operand::BranchThumbOffset(value) => value as u32,
            _ => unimplemented!(),
        }
    }

    pub fn write(&mut self, operand: Operand, value: u32) {
        match operand {
            Operand::Reg(reg) => self.cpu.regs[reg.number() as usize] = value,
            _ => unreachable!(),
        }
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
            self.execute(instruction);
        }
    }

    pub fn execute(&mut self, instruction: Instruction) {
        match instruction.opcode {
            Opcode::BKPT => {
                /*TODO BKPT */
                return;
            }
            Opcode::CBNZ | Opcode::CBZ => {
                let nonzero = instruction.opcode == Opcode::CBNZ;
                let n = self.read(instruction.operands[0]);
                let m = self.read(instruction.operands[1]); //i32
                if nonzero != (n == 0) {
                    self.branch_write_pc(self.cpu.regs[PC_INDEX] + m);
                }
                return;
            }
            _ => {}
        }
        if !self.condition_passed(instruction.condition) {
            return;
        }
        match instruction.opcode {
            Opcode::ADC | Opcode::ADD | Opcode::AND | Opcode::ASR | Opcode::BIC => {
                let d;
                let n;
                let m;
                if let Operand::Nothing = instruction.operands[2] {
                    // Opcode <Rdn>, <Rm>的形式
                    d = instruction.operands[0];
                    n = instruction.operands[0];
                    m = instruction.operands[1];
                } else {
                    d = instruction.operands[0];
                    n = instruction.operands[1];
                    m = instruction.operands[2];
                }
                let n = self.read(n);
                let m = self.read(m);

                let (result, carry, overflow) = match instruction.opcode {
                    Opcode::ADC | Opcode::ADD => add_with_carry(
                        n,
                        m,
                        if let Opcode::ADC = instruction.opcode {
                            self.cpu.apsr().c()
                        } else {
                            false
                        },
                    ),
                    Opcode::AND => (n & m, false, self.cpu.apsr().v()), //TODO carry
                    Opcode::ASR => {
                        let (result, carry) = shift_c(n, ShiftStyle::ASR, m, self.cpu.apsr().c());
                        (result, carry, self.cpu.apsr().v())
                    }
                    Opcode::BIC => (n & !m, false, self.cpu.apsr().v()), //TODO carry
                    _ => unreachable!(),
                };
                let Operand::Reg(reg) = d else {
                    unreachable!();
                };
                let reg_index = reg.number() as usize;
                if reg_index == PC_INDEX {
                    //Can only occur for ARM encoding
                    self.alu_write_pc(result); // setflags is always FALSE here
                } else {
                    self.write(d, result);
                }
                if instruction.s {
                    let mut apsr = self.cpu.apsr_mut();
                    apsr.set_n(result >> 31 & 1 == 1);
                    apsr.set_z(result != 0);
                    apsr.set_c(carry);
                    apsr.set_v(overflow);
                }
            }
            Opcode::ADR => {
                let d = instruction.operands[0];
                let n = instruction.operands[1];
                let result = self.align(self.cpu.regs[PC_INDEX], 4) + self.read(n);
                let Operand::Reg(reg) = d else {
                    unreachable!();
                };
                let reg_index = reg.number() as usize;
                if reg_index == PC_INDEX {
                    self.alu_write_pc(result);
                } else {
                    self.write(d, result);
                }
            }
            Opcode::B => {
                let imm32 = self.read(instruction.operands[0]); //i32
                self.branch_write_pc(self.cpu.regs[PC_INDEX] + imm32);
            }
            Opcode::BFC => {
                //将Rd的lsbit..msbit部分清0
                //operands[1]是将msbit作为寄存器索引了
                let d = instruction.operands[0];
                let lsbit = self.read(instruction.operands[2]);
                let msbit = self.read(instruction.operands[3]);
                if msbit >= lsbit {
                    let bits = self.read(d);
                    let width = (msbit - lsbit + 1) as u32;
                    let mask = ((1 << width) - 1) << lsbit;
                    let bits = bits & !mask;
                    self.write(d, bits);
                }
            }
            Opcode::BFI => {
                //将Rd的lsbit..msbit部分用Rn的0..(msbit-lsbit)替换
                let d = instruction.operands[0];
                let n = instruction.operands[1];
                let lsbit = self.read(instruction.operands[2]);
                let msbit = self.read(instruction.operands[3]);
                if msbit >= lsbit {
                    let bits = self.read(d);
                    let width = (msbit - lsbit + 1) as u32;
                    let mask = ((1 << width) - 1) << lsbit;
                    let bits = bits & (!mask | self.read(n) << lsbit);
                    self.write(d, bits);
                }
            }
            Opcode::BKPT => unreachable!(),
            Opcode::BL | Opcode::BLX => match instruction.operands[0] {
                Operand::BranchThumbOffset(imm32) => {
                    let imm32 = imm32 as u32; //i32
                    if let InstrSet::Arm = self.current_instr_set() {
                        self.cpu.regs[LR_INDEX] = self.cpu.regs[PC_INDEX] - 4;
                    } else {
                        self.cpu.regs[LR_INDEX] = self.cpu.regs[PC_INDEX] | 1;
                    }
                    let target_instr_set = InstrSet::Arm; //TODO 确定target_instr_set
                    let target_address;
                    if let InstrSet::Arm = target_instr_set {
                        target_address = self.align(self.cpu.regs[PC_INDEX], 4) + imm32;
                    } else {
                        target_address = self.cpu.regs[PC_INDEX] + imm32;
                    }
                    self.select_instr_set(target_instr_set);
                    self.branch_write_pc(target_address);
                }
                Operand::Reg(reg) => {
                    let target = self.cpu.regs[reg.number() as usize];
                    if let InstrSet::Arm = self.current_instr_set() {
                        self.cpu.regs[LR_INDEX] = self.cpu.regs[PC_INDEX] - 4;
                    } else {
                        self.cpu.regs[LR_INDEX] = (self.cpu.regs[PC_INDEX] - 2) | 1;
                    }
                    self.bw_write_pc(target);
                }
                _ => unreachable!(),
            },
            Opcode::BX => self.bw_write_pc(self.read(instruction.operands[0])),
            Opcode::BXJ => unimplemented!(), //跳转到Jazelle状态, 但目前只支持Arm和Thumb
            Opcode::CBNZ | Opcode::CBZ => unreachable!(),
            Opcode::CDP2(_coproc, _opc1, _opc2) => unimplemented!(), //TODO 协处理器
            Opcode::CLREX => unimplemented!(),                       //TODO CLREX 特权指令
            Opcode::CLZ => {
                let d = instruction.operands[0];
                let m = self.read(instruction.operands[1]);
                self.write(d, m.leading_zeros());
            }
            Opcode::CMN | Opcode::CMP => {
                let n = self.read(instruction.operands[0]);
                let m = self.read(instruction.operands[1]);
                let (result, carry, overflow) = match instruction.opcode {
                    Opcode::CMN => add_with_carry(n, m, false),
                    Opcode::CMP => add_with_carry(n, !m, true),
                    _ => unreachable!(),
                };
                let mut apsr = self.cpu.apsr_mut();
                apsr.set_n(result >> 31 & 1 == 1);
                apsr.set_z(result != 0);
                apsr.set_c(carry);
                apsr.set_v(overflow);
            }
            Opcode::CPS(_im) => todo!(),     //TODO CPS P1964 P1966
            Opcode::CPS_modeonly => todo!(), //TODO
            _ => unimplemented!(),
        }
    }
}
