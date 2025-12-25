use crate::cpu::{CPU, InstrSet};
use crate::instruction::{Condition, Instruction, InstructionKind, Operand};
use crate::memory::Memory;

pub struct Machine {
    pub cpu: CPU,
    pub arch_version: u32,
    pub memory: Memory,
}

impl Default for Machine {
    fn default() -> Self {
        Machine {
            cpu: CPU::default(),
            arch_version: 7,
            memory: Memory::default(),
        }
    }
}

impl Machine {
    /* P2639
    IsZero(x) = (BitCount(x) == 0)
    IsOnes(x) = (BitCount(x) == Len(x))
    IsZeroBit(x) = if IsZero(x) then '1' else '0'
    IsOnesBit(x) = if IsOnes(x) then '1' else '0'
    */
    pub fn execute(&mut self, instruction: Instruction) {
        if !self.condition_passed(instruction.cond) {
            return;
        }
        match instruction.kind {
            InstructionKind::ADC => {
                let (result, carry, overflow) = self.add_with_carry(
                    self.read(instruction.operand1),
                    self.read(instruction.operand2.unwrap()),
                    self.cpu.apsr().c(),
                );
                let Operand::Register { reg_index } = instruction.dest else {
                    unreachable!();
                };
                if reg_index == self.cpu.pc_index {
                    //Can only occur for ARM encoding
                    self.alu_write_pc(result); // setflags is always FALSE here
                } else {
                    self.write(instruction.dest, result);
                }
                if instruction.setflags {
                    let mut apsr = self.cpu.apsr_mut();
                    apsr.set_n(result >> 31 == 1);
                    apsr.set_z(result > 0);
                    apsr.set_c(carry);
                    apsr.set_v(overflow);
                }
            }
            InstructionKind::ADD => {
                let (result, carry, overflow) = self.add_with_carry(
                    self.read(instruction.operand1),
                    self.read(instruction.operand2.unwrap()),
                    false,
                );
                let Operand::Register { reg_index } = instruction.dest else {
                    unreachable!();
                };
                if reg_index == self.cpu.pc_index {
                    //Can only occur for ARM encoding
                    self.alu_write_pc(result); // setflags is always FALSE here
                } else {
                    self.write(instruction.dest, result);
                }
                if instruction.setflags {
                    let mut apsr = self.cpu.apsr_mut();
                    apsr.set_n(result >> 31 == 1);
                    apsr.set_z(result > 0);
                    apsr.set_c(carry);
                    apsr.set_v(overflow);
                }
            }
        }
    }

    pub fn run(&mut self) -> ! {
        loop {
            let instruction: u32 = 0xaf020000; //add r7, sp, #0x8
            let instruction = match match self.current_instr_set() {
                InstrSet::Arm => (self.parse_arm(instruction), 4),
                InstrSet::Thumb => {
                    let head = instruction >> 27;
                    if head == 0b11101 || head == 0b11110 || head == 0b11111 {
                        (self.parse_thumb2(instruction), 4)
                    } else {
                        (self.parse_thumb((instruction >> 16) as u16), 2)
                    }
                }
                _ => unimplemented!(),
            } {
                (Ok(t), pc_inc) => {
                    self.cpu.regs[self.cpu.pc_index] += pc_inc;
                    t
                }
                (Err(_), _) => todo!(), //TODO 处理非法的指令
            };
            self.execute(instruction);
        }
    }

    //P287
    pub fn condition_passed(&self, cond: Condition) -> bool {
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
        self.cpu.regs[self.cpu.pc_index] = address;
    }

    pub fn read(&self, operand: Operand) -> u32 {
        match operand {
            Operand::Immediate { value } => value,
            Operand::Register { reg_index } => self.cpu.regs[reg_index],
            Operand::RegWithShift { reg_index, shift } => {
                self.shift(self.cpu.regs[reg_index], shift, self.cpu.apsr().c())
            }
        }
    }

    pub fn write(&mut self, operand: Operand, value: u32) {
        match operand {
            Operand::Register { reg_index } => self.cpu.regs[reg_index] = value,
            _ => unreachable!(),
        }
    }
}
