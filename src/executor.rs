use bitfield::Bit;
use bitvec::prelude::*;
use yaxpeax_arm::armv7::{Instruction, Opcode, Operand, ShiftStyle};

use crate::arithmetic::*;
use crate::cpu::{InstrSet, LR_INDEX, PC_INDEX, SP_INDEX};
use crate::machine::Machine;
use crate::vmerror::VMError;

impl Machine {
    pub fn execute(&mut self, inst: Instruction) -> Result<(), VMError> {
        match inst.opcode {
            Opcode::BKPT => {
                //TODO BKPT
                return Ok(());
            }
            Opcode::CBNZ | Opcode::CBZ => {
                let nonzero = inst.opcode == Opcode::CBNZ;
                let n = self.read(inst.operands[0])?;
                let m = self.read(inst.operands[1])?; //i32
                if nonzero != (n == 0) {
                    self.branch_write_pc(self.cpu.regs[PC_INDEX] + m);
                }
                return Ok(());
            }
            _ => {}
        }
        if !self.condition_passed(inst.condition) {
            return Ok(());
        }
        match inst.opcode {
            Opcode::ADC
            | Opcode::ADD
            | Opcode::AND
            | Opcode::ASR
            | Opcode::BIC
            | Opcode::EOR
            | Opcode::LSL
            | Opcode::LSR
            | Opcode::MOV
            | Opcode::MUL
            | Opcode::MVN
            | Opcode::ORN
            | Opcode::ORR
            | Opcode::ROR
            | Opcode::RRX
            | Opcode::RSB
            | Opcode::RSC
            | Opcode::SBC
            | Opcode::SUB => {
                let d;
                let n;
                let m;
                if let Operand::Nothing = inst.operands[2] {
                    // Opcode <Rdn>, <Rm>的形式
                    d = inst.operands[0];
                    n = inst.operands[0];
                    m = inst.operands[1];
                } else {
                    d = inst.operands[0];
                    n = inst.operands[1];
                    m = inst.operands[2];
                }
                let n = self.read(n)?;
                let m = self.read(m)?;

                let (result, carry, overflow) = match inst.opcode {
                    Opcode::ADC => add_with_carry(n, m, self.cpu.apsr().c()),
                    Opcode::ADD => add_with_carry(n, m, false),
                    Opcode::AND => (n & m, false, self.cpu.apsr().v()), //TODO carry
                    Opcode::ASR => {
                        //如果m来自立即数, 那它也只有5位
                        let (result, carry) =
                            shift_c(n, ShiftStyle::ASR, m & 0xff, self.cpu.apsr().c());
                        (result, carry, self.cpu.apsr().v())
                    }
                    Opcode::BIC => (n & !m, false, self.cpu.apsr().v()), //TODO carry
                    Opcode::EOR => (n ^ m, false, self.cpu.apsr().v()),  //TODO carry
                    Opcode::LSL => {
                        //如果m来自立即数, 那它也只有5位
                        let (result, carry) =
                            shift_c(n, ShiftStyle::LSL, m & 0xff, self.cpu.apsr().c());
                        (result, carry, self.cpu.apsr().v())
                    }
                    Opcode::LSR => {
                        //如果m来自立即数, 那它也只有5位
                        let (result, carry) =
                            shift_c(n, ShiftStyle::LSR, m & 0xff, self.cpu.apsr().c());
                        (result, carry, self.cpu.apsr().v())
                    }
                    //MOV只有两个操作数, 所以根据前面的逻辑 d==n, m才是操作数
                    Opcode::MOV => (m, false, self.cpu.apsr().v()), //TODO carry
                    Opcode::MUL => (n * m, self.cpu.apsr().c(), self.cpu.apsr().v()),
                    //MVN只有两个操作数, 所以根据前面的逻辑 d==n, m才是操作数
                    Opcode::MVN => (!m, false, self.cpu.apsr().v()), //TODO carry
                    Opcode::ORN => (n | !m, false, self.cpu.apsr().v()), //TODO carry
                    Opcode::ORR => (n | m, false, self.cpu.apsr().v()), //TODO carry
                    Opcode::ROR => {
                        //如果m来自立即数, 那它也只有5位
                        let (result, carry) =
                            shift_c(n, ShiftStyle::ROR, m & 0xff, self.cpu.apsr().c());
                        (result, carry, self.cpu.apsr().v())
                    }
                    Opcode::RRX => {
                        //如果m来自立即数, 那它也只有5位
                        let (result, carry) = shift_c(m, ShiftStyle::ROR, 0, self.cpu.apsr().c());
                        (result, carry, self.cpu.apsr().v())
                    }
                    Opcode::RSB => add_with_carry(!n, m, false),
                    Opcode::RSC => add_with_carry(!n, m, self.cpu.apsr().c()),
                    Opcode::SBC => add_with_carry(n, !m, self.cpu.apsr().c()),
                    Opcode::SUB => add_with_carry(n, !m, true),
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
                    self.write(d, result)?;
                }
                //TODO InItBlock
                if inst.s {
                    let mut apsr = self.cpu.apsr_mut();
                    apsr.set_n(result >> 31 & 1 == 1);
                    apsr.set_z(result != 0);
                    apsr.set_c(carry);
                    apsr.set_v(overflow);
                }
            }
            Opcode::ADR => {
                let d = inst.operands[0];
                let n = inst.operands[1];
                let result = self.align(self.cpu.regs[PC_INDEX], 4) + self.read(n)?;
                let Operand::Reg(reg) = d else {
                    unreachable!();
                };
                let reg_index = reg.number() as usize;
                if reg_index == PC_INDEX {
                    self.alu_write_pc(result);
                } else {
                    self.write(d, result)?;
                }
            }
            Opcode::B => {
                let imm32 = self.read(inst.operands[0])?; //i32
                self.branch_write_pc(self.cpu.regs[PC_INDEX] + imm32);
            }
            Opcode::BFC => {
                //将Rd的lsbit..msbit部分清0
                //operands[1]是将msbit作为寄存器索引了
                let d = inst.operands[0];
                let lsbit = self.read(inst.operands[2])?;
                let msbit = self.read(inst.operands[3])?;
                if msbit >= lsbit {
                    let bits = self.read(d)?;
                    let width = (msbit - lsbit + 1) as u32;
                    let mask = ((1 << width) - 1) << lsbit;
                    let bits = bits & !mask;
                    self.write(d, bits)?;
                }
            }
            Opcode::BFI => {
                //将Rd的lsbit..msbit部分用Rn的0..(msbit-lsbit)替换
                let d = inst.operands[0];
                let n = inst.operands[1];
                let lsbit = self.read(inst.operands[2])?;
                let msbit = self.read(inst.operands[3])?;
                if msbit >= lsbit {
                    let bits = self.read(d)?;
                    let width = (msbit - lsbit + 1) as u32;
                    let mask = ((1 << width) - 1) << lsbit;
                    let bits = bits & (!mask | self.read(n)? << lsbit);
                    self.write(d, bits)?;
                }
            }
            Opcode::BKPT => unreachable!(),
            Opcode::BL | Opcode::BLX => match inst.operands[0] {
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
            Opcode::BX => self.bw_write_pc(self.read(inst.operands[0])?),
            Opcode::BXJ => unimplemented!(), //跳转到Jazelle状态, 但目前只支持Arm和Thumb
            Opcode::CBNZ | Opcode::CBZ => unreachable!(),
            Opcode::CDP2(..) => unimplemented!(), //TODO CDP2 协处理器
            Opcode::CLREX => unimplemented!(),    //TODO CLREX 特权指令
            Opcode::CLZ => {
                let d = inst.operands[0];
                let m = self.read(inst.operands[1])?;
                self.write(d, m.leading_zeros())?;
            }
            Opcode::CMN | Opcode::CMP => {
                let n = self.read(inst.operands[0])?;
                let m = self.read(inst.operands[1])?;
                let (result, carry, overflow) = match inst.opcode {
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
            Opcode::CPS(..) => unimplemented!(), //TODO CPS P1964 P1966
            Opcode::CPS_modeonly => unimplemented!(), //TODO
            Opcode::CSDB => unimplemented!(),    //TODO CSDB
            Opcode::DBG => unimplemented!(),     //TODO DBG
            Opcode::DMB => unimplemented!(),     //TODO DMB
            Opcode::DSB => unimplemented!(),     //TODO DSB
            Opcode::ENTERX => unimplemented!(),  //跳转到ThumbEE状态, 但目前只支持Arm和Thumb
            Opcode::ERET => unimplemented!(),    //TODO ERET
            Opcode::HINT => unimplemented!(),    //TODO HINT
            Opcode::HVC => unimplemented!(),     //TODO HVC
            Opcode::ISB => unimplemented!(),     //TODO ISB
            Opcode::IT => {
                let firstcond = self.read(inst.operands[0])?;
                let mask = self.read(inst.operands[1])?;
                self.cpu
                    .it_state_mut()
                    .set_value((firstcond << 4 | mask) as u8);
            }
            Opcode::Invalid => unimplemented!(),   //TODO Invalid
            Opcode::LDC(..) => unimplemented!(),   //TODO LDC
            Opcode::LDC2(..) => unimplemented!(),  //TODO LDC2
            Opcode::LDC2L(..) => unimplemented!(), //TODO LDC2L,
            Opcode::LDCL(..) => unimplemented!(),  //TODO LDCL
            Opcode::LDM(add, pre, _wback, _usermode) => {
                //TODO usermode
                let add = if add { 1 } else { -1i32 as u32 };
                let n = self.read(inst.operands[0])?;
                let mut address = n + if pre { 4 * add } else { 0 };
                let registers = self.read(inst.operands[1])?;
                for i in 0..16 {
                    if registers >> i & 1 != 1 {
                        continue;
                    }
                    if i != PC_INDEX {
                        self.cpu.regs[i] = self.read_memory_word(address)?;
                    } else {
                        self.load_write_pc(self.read_memory_word(address)?);
                    }
                    address += 4 * add;
                }
                //inst.operands[0]一定是RegWBack
                self.write(inst.operands[0], address)?;
            }
            Opcode::LDR
            | Opcode::LDRB
            | Opcode::LDRBT
            | Opcode::LDRH
            | Opcode::LDRHT
            | Opcode::LDRSB
            | Opcode::LDRSBT
            | Opcode::LDRSH
            | Opcode::LDRSHT
            | Opcode::LDRT => {
                //TODO LDRBT LDRHT LDRSBT LDRSHT LDRT
                let t = inst.operands[0];
                let address = self.read_address(inst.operands[1])?;
                let mut word = self.read_memory_word(address)?;
                match inst.opcode {
                    Opcode::LDRB | Opcode::LDRBT => word = word & 0xff,
                    Opcode::LDRH | Opcode::LDRHT => word = word & 0xffff,
                    Opcode::LDRSB | Opcode::LDRSBT => word = (word & 0xff) as i8 as i32 as u32,
                    Opcode::LDRSH | Opcode::LDRSHT => word = (word & 0xffff) as i16 as i32 as u32,
                    _ => {}
                }
                let Operand::Reg(reg) = t else { unreachable!() };
                //TODO 对齐检查
                if reg.number() as usize == PC_INDEX {
                    self.load_write_pc(word);
                } else {
                    self.write(t, word)?;
                }
                self.write(inst.operands[1], address)?;
            }
            Opcode::LDRD => {
                let t = inst.operands[0];
                let t2 = inst.operands[1];
                let address = self.read_address(inst.operands[2])?;
                self.write(t, self.read_memory_word(address)?)?;
                self.write(t2, self.read_memory_word(address + 4)?)?;
                self.write(inst.operands[2], address)?;
            }
            Opcode::LDREX => unimplemented!(),     //TODO LDREX
            Opcode::LDREXB => unimplemented!(),    //TODO LDREXB
            Opcode::LDREXD => unimplemented!(),    //TODO LDREXD
            Opcode::LDREXH => unimplemented!(),    //TODO LDREXH
            Opcode::LEAVEX => {}                   //跳转到Thumb状态, 但目前只支持Arm和Thumb
            Opcode::MCR2(..) => unimplemented!(),  //TODO MCR2
            Opcode::MCRR(..) => unimplemented!(),  //TODO MCRR
            Opcode::MCRR2(..) => unimplemented!(), //TODO MCRR2
            Opcode::MLA => {
                let d = inst.operands[0];
                let n = self.read(inst.operands[1])?;
                let m = self.read(inst.operands[2])?;
                let a = self.read(inst.operands[3])?;
                let result = n * m + a;
                self.write(d, result)?;
                if inst.s {
                    let mut apsr = self.cpu.apsr_mut();
                    apsr.set_n(result >> 31 & 1 == 1);
                    apsr.set_z(result != 0);
                }
            }
            Opcode::MLS => {
                let d = inst.operands[0];
                let n = self.read(inst.operands[1])?;
                let m = self.read(inst.operands[2])?;
                let a = self.read(inst.operands[3])?;
                let result = a - n * m;
                self.write(d, result)?;
            }
            Opcode::MOVT => {
                let d = inst.operands[0];
                let imm16 = self.read(inst.operands[1])?;
                self.write(d, self.read(d)? & (imm16 << 16 | 0xffff))?;
            }
            Opcode::MRC2(..) => unimplemented!(),  //TODO MRC2
            Opcode::MRRC(..) => unimplemented!(),  //TODO MRRC
            Opcode::MRRC2(..) => unimplemented!(), //TODO MRRC2
            //TODO MRS banked register
            Opcode::MRS => self.write(inst.operands[0], self.read(inst.operands[1])?)?,
            //TODO MSR banked register
            Opcode::MSR => self.write(inst.operands[0], self.read(inst.operands[1])?)?,
            Opcode::NOP => {}
            Opcode::PKHBT => {
                let d = inst.operands[0];
                let n = self.read(inst.operands[1])?;
                let m = self.read(inst.operands[2])?;
                self.write(d, n & 0xffff | m & 0xffff0000)?;
            }
            Opcode::PKHTB => {
                let d = inst.operands[0];
                let n = self.read(inst.operands[1])?;
                let m = self.read(inst.operands[2])?;
                self.write(d, n & 0xffff0000 | m & 0xffff)?;
            }
            Opcode::PLD => unimplemented!(), //TODO PLD
            Opcode::PLI => unimplemented!(), //TODO PLI
            Opcode::POP => {
                let mut address = self.cpu.regs[SP_INDEX];
                let registers = self.read(inst.operands[0])?;
                //TODO 对齐
                for i in 0..16 {
                    if registers >> i & 1 != 1 {
                        continue;
                    }
                    if i != PC_INDEX {
                        self.cpu.regs[i] = self.read_memory_word(address)?;
                    } else {
                        self.load_write_pc(self.read_memory_word(address)?);
                    }
                    address += 4;
                }
                self.cpu.regs[SP_INDEX] = address;
            }
            Opcode::PUSH => {
                let mut address = self.cpu.regs[SP_INDEX];
                let registers = self.read(inst.operands[0])?;
                //TODO 对齐
                for i in (0..16).rev() {
                    if registers >> i & 1 != 1 {
                        continue;
                    }
                    address -= 4;
                    self.write_memory_word(address, self.cpu.regs[i])?;
                }
                self.cpu.regs[SP_INDEX] = address;
            }
            Opcode::QADD => unimplemented!(), //TODO QADD
            Opcode::QADD16 | Opcode::UQADD16 => unimplemented!(), //TODO QADD16 UQADD16
            Opcode::QADD8 | Opcode::UQADD8 => unimplemented!(), //TODO QADD8 UQADD8
            Opcode::QASX | Opcode::UQASX => unimplemented!(), //TODO QASX UQASX
            Opcode::QDADD => unimplemented!(), //TODO QDADD
            Opcode::QDSUB => unimplemented!(), //TODO QDSUB
            Opcode::QSAX | Opcode::UQSAX => unimplemented!(), //TODO QSAX UQSAX
            Opcode::QSUB => unimplemented!(), //TODO QSUB
            Opcode::QSUB16 | Opcode::UQSUB16 => unimplemented!(), //TODO QSUB16 UQSUB16
            Opcode::QSUB8 | Opcode::UQSUB8 => unimplemented!(), //TODO QSUB8 UQSUB8
            Opcode::RBIT => {
                let d = inst.operands[0];
                let m = self.read(inst.operands[1])?;
                self.write(d, m.reverse_bits())?;
            }
            Opcode::REV => {
                let d = inst.operands[0];
                let m = self.read(inst.operands[1])?;
                self.write(d, u32::from_be_bytes(m.to_le_bytes()))?;
            }
            Opcode::REV16 => {
                let d = inst.operands[0];
                let m = self.read(inst.operands[1])?;
                let bytes = m.to_le_bytes();
                self.write(
                    d,
                    u32::from_le_bytes([bytes[1], bytes[0], bytes[3], bytes[2]]),
                )?;
            }
            Opcode::REVSH => {
                let d = inst.operands[0];
                let m = self.read(inst.operands[1])?;
                let bytes = m.to_le_bytes();
                let low = bytes[0] as i8 as i32 as u32;
                let high = bytes[1] as u32;
                self.write(d, low << 8 | high)?;
            }
            Opcode::RFE(..) => unimplemented!(), //TODO RFE
            Opcode::SADD16 | Opcode::UADD16 => {
                let d = inst.operands[0];
                let n = self.read(inst.operands[1])?;
                let m = self.read(inst.operands[2])?;
                let sum1 = (n & 0xffff) + (m & 0xffff);
                let sum2 = (n >> 16) + (m >> 16);
                self.write(d, sum2 << 16 | (sum1 & 0xffff))?;
                self.cpu.apsr_mut().set_ge(
                    if match inst.opcode {
                        Opcode::SADD16 => sum2 as i32 >= 0,
                        Opcode::UADD16 => sum2 >= 0x10000,
                        _ => unreachable!(),
                    } {
                        0b11
                    } else {
                        0b00
                    } << 2
                        | if match inst.opcode {
                            Opcode::SADD16 => sum1 as i32 >= 0,
                            Opcode::UADD16 => sum1 >= 0x10000,
                            _ => unreachable!(),
                        } {
                            0b11
                        } else {
                            0b00
                        },
                );
            }
            Opcode::SADD8 | Opcode::UADD8 => {
                let d = inst.operands[0];
                let n = self.read(inst.operands[1])?.to_le_bytes();
                let m = self.read(inst.operands[2])?.to_le_bytes();
                let mut ge = 0;
                let mut sum = [0; 4];
                for i in 0..4 {
                    sum[i] = n[i] + m[i];
                    let sum = n[i] as u32 + m[i] as u32;
                    for i in 0..4 {
                        if match inst.opcode {
                            Opcode::SADD8 => sum as i8 >= 0,
                            Opcode::UADD8 => sum >= 0x100,
                            _ => unreachable!(),
                        } {
                            ge |= 1 << i;
                        }
                    }
                }
                self.write(d, u32::from_le_bytes(sum))?;
                self.cpu.apsr_mut().set_ge(ge);
            }
            Opcode::SASX | Opcode::UASX => {
                let d = inst.operands[0];
                let n = self.read(inst.operands[1])?;
                let m = self.read(inst.operands[2])?;
                let diff = (n & 0xffff) - (m >> 16);
                let sum = (n >> 16) + (n & 0xffff);
                self.write(d, sum << 16 | (diff & 0xffff))?;
                self.cpu.apsr_mut().set_ge(
                    if match inst.opcode {
                        Opcode::SASX => sum as i32 >= 0,
                        Opcode::UASX => sum >= 0x10000,
                        _ => unreachable!(),
                    } {
                        0b11
                    } else {
                        0b00
                    } << 2
                        | if diff as i32 >= 0 { 0b11 } else { 0b00 },
                );
            }
            Opcode::SBFX | Opcode::UBFX => {
                let d = inst.operands[0];
                let n = self.read(inst.operands[1])?;
                let lsb = self.read(inst.operands[2])? as usize;
                let width = self.read(inst.operands[3])? as usize + 1;
                let msb = lsb + width;
                self.write(
                    d,
                    n.view_bits::<Lsb0>().get(lsb..msb).unwrap().load::<i32>() as u32,
                )?;
            }
            Opcode::SDIV | Opcode::UDIV => unimplemented!(), //TODO SDIV UDIV
            Opcode::SEL => {
                let d = inst.operands[0];
                let n = self.read(inst.operands[1])?.to_le_bytes();
                let m = self.read(inst.operands[2])?.to_le_bytes();
                let mut sum = [0; 4];
                let ge = self.cpu.apsr().ge();
                for i in 0..4 {
                    sum[i] = if ge.bit(i) == true { n[i] } else { m[i] };
                }
                self.write(d, u32::from_le_bytes(sum))?;
            }
            Opcode::SETEND => unimplemented!(), //TODO SETEND
            Opcode::SEV => unimplemented!(),    //TODO SEV
            Opcode::SHADD16 | Opcode::UHADD16 => {
                let d = inst.operands[0];
                let n = self.read(inst.operands[1])?;
                let m = self.read(inst.operands[2])?;
                let sum1 = (n & 0xffff) + (m & 0xffff);
                let sum2 = (n >> 16) + (m >> 16);
                self.write(d, ((sum2 >> 1 & 0xffff) << 16) | (sum1 >> 1 & 0xffff))?;
            }
            Opcode::SHADD8 | Opcode::UHADD8 => {
                let d = inst.operands[0];
                let n = self.read(inst.operands[1])?.to_le_bytes();
                let m = self.read(inst.operands[2])?.to_le_bytes();
                let mut sum = [0; 4];
                for i in 0..4 {
                    sum[i] = ((n[i] as i32 + m[i] as i32) >> 1 & 0xff) as u8;
                }
                self.write(d, u32::from_le_bytes(sum))?;
            }
            Opcode::SHASX | Opcode::UHASX => {
                let d = inst.operands[0];
                let n = self.read(inst.operands[1])?;
                let m = self.read(inst.operands[2])?;
                let diff = (n & 0xffff) - (m >> 16);
                let sum = (n >> 16) + (n & 0xffff);
                self.write(d, ((sum >> 1 & 0xffff) << 16) | (diff >> 1 & 0xffff))?;
            }
            Opcode::SHSAX | Opcode::UHSAX => {
                let d = inst.operands[0];
                let n = self.read(inst.operands[1])?;
                let m = self.read(inst.operands[2])?;
                let sum = (n & 0xffff) + (m >> 16);
                let diff = (n >> 16) - (n & 0xffff);
                self.write(d, ((diff >> 1 & 0xffff) << 16) | (sum >> 1 & 0xffff))?;
            }
            Opcode::SHSUB16 | Opcode::UHSUB16 => {
                let d = inst.operands[0];
                let n = self.read(inst.operands[1])?;
                let m = self.read(inst.operands[2])?;
                let diff1 = (n & 0xffff) - (m & 0xffff);
                let diff2 = (n >> 16) - (n >> 16);
                self.write(d, ((diff2 >> 1 & 0xffff) << 16) | (diff1 >> 1 & 0xffff))?;
            }
            Opcode::SHSUB8 | Opcode::UHSUB8 => {
                let d = inst.operands[0];
                let n = self.read(inst.operands[1])?.to_le_bytes();
                let m = self.read(inst.operands[2])?.to_le_bytes();
                let mut diff = [0; 4];
                for i in 0..4 {
                    diff[i] = match inst.opcode {
                        Opcode::SHADD8 => ((n[i] as i32 - m[i] as i32) >> 1 & 0xff) as u8,
                        Opcode::UHSUB8 => ((n[i] - m[i]) >> 1 & 0xff) as u8,
                        _ => unreachable!(),
                    }
                }
                self.write(d, u32::from_le_bytes(diff))?;
            }
            Opcode::SMAL(..) => unimplemented!(), //TODO SMAL
            Opcode::SMC => unimplemented!(),      //TODO SMC
            Opcode::SMLA(n_high, m_high) => {
                let d = inst.operands[0];
                let n = self.read(inst.operands[1])?;
                let m = self.read(inst.operands[2])?;
                let a = self.read(inst.operands[3])? as i64;
                let operand1 = if n_high { n >> 16 } else { n & 0xffff } as i64;
                let operand2 = if m_high { m >> 16 } else { m & 0xffff } as i64;
                let result = operand1 * operand2 + a;
                self.write(d, result as u32)?;
                if result >> 32 != 0 {
                    self.cpu.apsr_mut().set_q(true);
                }
            }
            Opcode::SMLAD => {
                let m_swap = false; //TODO m_swap
                let d = inst.operands[0];
                let n = self.read(inst.operands[1])? as i64;
                let m = self.read(inst.operands[2])?;
                let a = self.read(inst.operands[3])? as i64;
                let operand2 = if m_swap { rotate_right(m, 16) } else { m } as i64;
                let product1 = (n & 0xffff) * (operand2 & 0xffff);
                let product2 = (n >> 16) * (operand2 >> 16);
                let result = product1 + product2 + a;
                self.write(d, result as u32)?;
                if result >> 32 != 0 {
                    self.cpu.apsr_mut().set_q(true);
                }
            }
            Opcode::SMLAL | Opcode::UMLAL => {
                let dlo = inst.operands[0];
                let dhi = inst.operands[1];
                let n = self.read(inst.operands[2])?;
                let m = self.read(inst.operands[3])?;
                let result = match inst.opcode {
                    Opcode::SMLAL => {
                        (n as i64 * m as i64
                            + ((self.read(dhi)? as i64) << 32 | self.read(dlo)? as i64))
                            as u64
                    }
                    Opcode::UMLAL => {
                        n as u64 * m as u64
                            + ((self.read(dhi)? as u64) << 32 | self.read(dlo)? as u64)
                    }
                    _ => unreachable!(),
                };
                self.write(dlo, (result & 0xffffffff) as u32)?;
                self.write(dhi, (result >> 32) as u32)?;
                if inst.s {
                    let mut apsr = self.cpu.apsr_mut();
                    apsr.set_n(result >> 63 & 1 == 1);
                    apsr.set_z(result > 0);
                }
            }
            Opcode::SMLALD(m_swap) => {
                let dlo = inst.operands[0];
                let dhi = inst.operands[1];
                let n = self.read(inst.operands[2])? as i64;
                let m = self.read(inst.operands[3])?;
                let operand2 = if m_swap { rotate_right(m, 16) } else { m } as i64;
                let product1 = (n & 0xffff) * (operand2 & 0xffff);
                let product2 = (n >> 16) * (operand2 >> 16);
                let result = product1
                    + product2
                    + ((self.read(dhi)? as u64) << 32 | self.read(dlo)? as u64) as i64;
                self.write(dlo, (result & 0xffffffff) as u32)?;
                self.write(dhi, (result >> 32) as u32)?;
            }
            Opcode::SMLAL_halfword(..) => unimplemented!(), //TODO SMLAL_halfword
            Opcode::SMLAW(m_high) => {
                let d = inst.operands[0];
                let n = self.read(inst.operands[1])? as i64;
                let m = self.read(inst.operands[2])?;
                let a = self.read(inst.operands[3])? as i64;
                let operand2 = if m_high { m >> 16 } else { m & 0xffff } as i64;
                let result = n * operand2 + (a << 16);
                self.write(d, ((result >> 16) & 0xffffffff) as u32)?;
                if result >> 48 != 0 {
                    self.cpu.apsr_mut().set_q(true);
                }
            }
            Opcode::SMLSD => {
                let m_swap = false; //TODO m_swap
                let d = inst.operands[0];
                let n = self.read(inst.operands[1])? as i64;
                let m = self.read(inst.operands[2])?;
                let a = self.read(inst.operands[3])? as i64;
                let operand2 = if m_swap { rotate_right(m, 16) } else { m } as i64;
                let product1 = (n & 0xffff) * (operand2 & 0xffff);
                let product2 = (n >> 16) * (operand2 >> 16);
                let result = product1 - product2 + a;
                self.write(d, result as u32)?;
                if result >> 32 != 0 {
                    self.cpu.apsr_mut().set_q(true);
                }
            }
            Opcode::SMLSLD(m_swap) => {
                let dlo = inst.operands[0];
                let dhi = inst.operands[1];
                let n = self.read(inst.operands[2])? as i64;
                let m = self.read(inst.operands[3])?;
                let operand2 = if m_swap { rotate_right(m, 16) } else { m } as i64;
                let product1 = (n & 0xffff) * (operand2 & 0xffff);
                let product2 = (n >> 16) * (operand2 >> 16);
                let result = product1 - product2
                    + ((self.read(dhi)? as u64) << 32 | self.read(dlo)? as u64) as i64;
                self.write(dlo, (result & 0xffffffff) as u32)?;
                self.write(dhi, (result >> 32) as u32)?;
            }
            Opcode::SMMLA => {
                let round = false; //TODO round
                let d = inst.operands[0];
                let n = self.read(inst.operands[1])? as i64;
                let m = self.read(inst.operands[2])? as i64;
                let a = self.read(inst.operands[3])? as i64;
                let mut result = (a << 32) + n * m;
                if round {
                    result += 0x80000000;
                }
                self.write(d, (result >> 32) as u32)?;
            }
            Opcode::SMMLS => {
                let round = false; //TODO round
                let d = inst.operands[0];
                let n = self.read(inst.operands[1])? as i64;
                let m = self.read(inst.operands[2])? as i64;
                let a = self.read(inst.operands[3])? as i64;
                let mut result = (a << 32) - n * m;
                if round {
                    result += 0x80000000;
                }
                self.write(d, (result >> 32) as u32)?;
            }
            Opcode::SMMUL => {
                let round = false; //TODO round
                let d = inst.operands[0];
                let n = self.read(inst.operands[1])? as i64;
                let m = self.read(inst.operands[2])? as i64;
                let mut result = n * m;
                if round {
                    result += 0x80000000;
                }
                self.write(d, (result >> 32) as u32)?;
            }
            Opcode::SMUAD => {
                let m_swap = false; //TODO m_swap
                let d = inst.operands[0];
                let n = self.read(inst.operands[1])? as i64;
                let m = self.read(inst.operands[2])?;
                let operand2 = if m_swap { rotate_right(m, 16) } else { m } as i64;
                let product1 = (n & 0xffff) * (operand2 & 0xffff);
                let product2 = (n >> 16) * (operand2 >> 16);
                let result = product1 + product2;
                self.write(d, result as u32)?;
                if result >> 32 != 0 {
                    self.cpu.apsr_mut().set_q(true);
                }
            }
            Opcode::SMUL(n_high, m_high) => {
                let d = inst.operands[0];
                let n = self.read(inst.operands[1])?;
                let m = self.read(inst.operands[2])?;
                let operand1 = if n_high { n >> 16 } else { n & 0xffff } as i64;
                let operand2 = if m_high { m >> 16 } else { m & 0xffff } as i64;
                let result = operand1 * operand2;
                self.write(d, result as u32)?;
            }
            Opcode::SMULL | Opcode::UMULL => {
                let dlo = inst.operands[0];
                let dhi = inst.operands[1];
                let n = self.read(inst.operands[1])?;
                let m = self.read(inst.operands[2])?;
                let result = match inst.opcode {
                    Opcode::SMULL => (n as i64 * m as i64) as u64,
                    Opcode::UMULL => n as u64 * m as u64,
                    _ => unreachable!(),
                };
                self.write(dlo, (result & 0xffffffff) as u32)?;
                self.write(dhi, (result >> 32) as u32)?;
                if inst.s {
                    let mut apsr = self.cpu.apsr_mut();
                    apsr.set_n(result >> 63 & 1 == 1);
                    apsr.set_z(result > 0);
                }
            }
            Opcode::SMULW(m_high) => {
                let d = inst.operands[0];
                let n = self.read(inst.operands[1])? as i64;
                let m = self.read(inst.operands[2])?;
                let operand2 = if m_high { m >> 16 } else { m & 0xffff } as i64;
                let result = n * operand2;
                self.write(d, ((result >> 16) & 0xffffffff) as u32)?;
            }
            Opcode::SMUSD => {
                let m_swap = false; //TODO m_swap
                let d = inst.operands[0];
                let n = self.read(inst.operands[1])? as i64;
                let m = self.read(inst.operands[2])?;
                let operand2 = if m_swap { rotate_right(m, 16) } else { m } as i64;
                let product1 = (n & 0xffff) * (operand2 & 0xffff);
                let product2 = (n >> 16) * (operand2 >> 16);
                let result = product1 - product2;
                self.write(d, result as u32)?;
            }
            Opcode::SRS(..) => unimplemented!(), //TODO SRS
            Opcode::SSAT | Opcode::USAT => unimplemented!(), //TODO SSAT USAT
            Opcode::SSAT16 | Opcode::USAT16 => unimplemented!(), //TODO SSAT16 USAT16
            Opcode::SSAX | Opcode::USAX => {
                let d = inst.operands[0];
                let n = self.read(inst.operands[1])?;
                let m = self.read(inst.operands[2])?;
                let sum = (n & 0xffff) + (m >> 16);
                let diff = (n >> 16) - (n & 0xffff);
                self.write(d, (diff << 16) | (sum & 0xffff))?;
                self.cpu.apsr_mut().set_ge(
                    if diff as i32 >= 0 { 0b11 } else { 0b00 } << 2
                        | if match inst.opcode {
                            Opcode::SSAX => sum as i32 >= 0,
                            Opcode::USAX => sum >= 0x10000,
                            _ => unreachable!(),
                        } {
                            0b11
                        } else {
                            0b00
                        },
                );
            }
            Opcode::SSUB16 | Opcode::USUB16 => {
                let d = inst.operands[0];
                let n = self.read(inst.operands[1])?;
                let m = self.read(inst.operands[2])?;
                let diff1 = (n & 0xffff) - (m & 0xffff);
                let diff2 = (n >> 16) - (n >> 16);
                self.write(d, ((diff2 >> 1 & 0xffff) << 16) | (diff1 >> 1 & 0xffff))?;
                self.cpu.apsr_mut().set_ge(
                    (if diff2 as i32 >= 0 { 0b11 } else { 0b00 }) << 2
                        | if diff1 as i32 >= 0 { 0b11 } else { 0b00 },
                );
            }
            Opcode::SSUB8 | Opcode::USUB8 => {
                let d = inst.operands[0];
                let n = self.read(inst.operands[1])?.to_le_bytes();
                let m = self.read(inst.operands[2])?.to_le_bytes();
                let mut diff = [0; 4];
                let mut ge = 0;
                for i in 0..4 {
                    let x = n[i] as i32 - m[i] as i32;
                    diff[i] = x as u8;
                    for i in 0..4 {
                        if x >= 0 {
                            ge |= 1 << i;
                        }
                    }
                }
                self.write(d, u32::from_le_bytes(diff))?;
                self.cpu.apsr_mut().set_ge(ge);
            }
            Opcode::STC(..) => unimplemented!(),   //TODO STC
            Opcode::STC2(..) => unimplemented!(),  //TODO STC2
            Opcode::STC2L(..) => unimplemented!(), //TODO STC2L
            Opcode::STCL(..) => unimplemented!(),  //TODO STCL
            Opcode::STM(add, pre, _wback, _usermode) => {
                //TODO usermode
                let add = if add { 1 } else { -1i32 as u32 };
                let n = self.read(inst.operands[0])?;
                let mut address = n + if pre { 4 * add } else { 0 };
                let registers = self.read(inst.operands[1])?;
                for i in 0..16 {
                    if registers >> i & 1 != 1 {
                        continue;
                    }
                    self.write_memory_word(address, self.cpu.regs[i])?;
                    address += 4 * add;
                }
                //inst.operands[0]一定是RegWBack
                self.write(inst.operands[0], address)?;
            }
            Opcode::STR
            | Opcode::STRB
            | Opcode::STRBT
            | Opcode::STRH
            | Opcode::STRHT
            | Opcode::STRT => {
                //TODO STRBT STRHT STRT
                let t = inst.operands[0];
                let address = self.read_address(inst.operands[1])?;
                let word = self.read(t)?;
                //TODO 对齐检查
                match inst.opcode {
                    Opcode::STR | Opcode::STRT => self.write_memory_word(address, word)?,
                    Opcode::STRB | Opcode::STRBT => {
                        self.write_memory(address, (word & 0xff) as u8)?
                    }
                    Opcode::STRH | Opcode::STRHT => {
                        self.write_memory_halfword(address, word as u16)?
                    }
                    _ => {}
                }
                self.write(inst.operands[1], address)?;
            }
            Opcode::STRD => {
                let t = self.read(inst.operands[0])?;
                let t2 = self.read(inst.operands[1])?;
                let address = self.read_address(inst.operands[2])?;
                self.write_memory_word(address, t)?;
                self.write_memory_word(address + 4, t2)?;
                self.write(inst.operands[2], address)?;
            }
            Opcode::STREX => unimplemented!(),  //TODO STREX
            Opcode::STREXB => unimplemented!(), //TODO STREXB
            Opcode::STREXD => unimplemented!(), //TODO STREXD
            Opcode::STREXH => unimplemented!(), //TODO STREXH
            Opcode::SVC => unimplemented!(),    //TODO SVC
            Opcode::SWP | Opcode::SWPB => {
                let t = inst.operands[0];
                let t2 = self.read(inst.operands[1])?;
                let n = self.read(inst.operands[2])?;
                if let Opcode::SWPB = inst.opcode {
                    let data = self.read_memory(n)? as u32;
                    self.write(t, data)?;
                    self.write_memory(n, t2 as u8)?;
                } else {
                    let data = self.read_memory_word(n)?;
                    self.write(t, rotate_right(data, 8 * (n & 0b11)))?;
                    self.write_memory_word(n, t2)?;
                };
            }
            Opcode::SXTAB
            | Opcode::SXTAB16
            | Opcode::SXTAH
            | Opcode::UXTAB
            | Opcode::UXTAB16
            | Opcode::UXTAH => {
                let d = inst.operands[0];
                let n = self.read(inst.operands[1])?;
                let m = self.read(inst.operands[2])?;
                let rotation = self.read(inst.operands[3])?;
                let rotated = rotate_right(m, rotation).to_le_bytes();
                self.write(
                    d,
                    match inst.opcode {
                        Opcode::SXTAB => n + rotated[0] as i8 as i32 as u32,
                        Opcode::SXTAB16 => {
                            ((n >> 16) + rotated[2] as i8 as i32 as u32) << 16
                                | ((n & 0xffff) + rotated[0] as i8 as i32 as u32)
                        }
                        Opcode::SXTAH => {
                            n + i16::from_le_bytes([rotated[0], rotated[1]]) as i32 as u32
                        }
                        Opcode::UXTAB => n + rotated[0] as u32,
                        Opcode::UXTAB16 => {
                            ((n >> 16) + rotated[2] as u32) << 16
                                | ((n & 0xffff) + rotated[0] as u32)
                        }
                        Opcode::UXTAH => n + u16::from_le_bytes([rotated[0], rotated[1]]) as u32,
                        _ => unreachable!(),
                    },
                )?;
            }
            Opcode::SXTB
            | Opcode::SXTB16
            | Opcode::SXTH
            | Opcode::UXTB
            | Opcode::UXTB16
            | Opcode::UXTH => {
                let d = inst.operands[0];
                let m = self.read(inst.operands[1])?;
                let rotation = self.read(inst.operands[2])?;
                let rotated = rotate_right(m, rotation).to_le_bytes();
                self.write(
                    d,
                    match inst.opcode {
                        Opcode::SXTB => rotated[0] as i8 as i32 as u32,
                        Opcode::SXTB16 => {
                            (rotated[2] as i8 as i32 as u32) << 16 | rotated[0] as i8 as i32 as u32
                        }
                        Opcode::SXTH => i16::from_le_bytes([rotated[0], rotated[1]]) as i32 as u32,
                        Opcode::UXTB => rotated[0] as u32,
                        Opcode::UXTB16 => (rotated[2] as u32) << 16 | rotated[0] as u32,
                        Opcode::UXTH => u16::from_le_bytes([rotated[0], rotated[1]]) as u32,
                        _ => unreachable!(),
                    },
                )?;
            }
            Opcode::TBB => {
                let address = self.read(inst.operands[0])?;
                let halfwords = self.read_memory(address)? as u32;
                self.branch_write_pc(self.cpu.regs[PC_INDEX] + 2 * halfwords);
            }
            Opcode::TBH => {
                let address = self.read(inst.operands[0])?;
                let halfwords = self.read_memory_halfword(address)? as u32;
                self.branch_write_pc(self.cpu.regs[PC_INDEX] + 2 * halfwords);
            }
            Opcode::TEQ => {
                let n = self.read(inst.operands[0])?;
                let m = self.read(inst.operands[1])?;
                let result = n ^ m;
                let mut apsr = self.cpu.apsr_mut();
                apsr.set_n(result >> 31 & 1 == 1);
                apsr.set_z(result != 0);
                apsr.set_c(false); //TODO carry
            }
            Opcode::TST => {
                let n = self.read(inst.operands[0])?;
                let m = self.read(inst.operands[1])?;
                let result = n & m;
                let mut apsr = self.cpu.apsr_mut();
                apsr.set_n(result >> 31 & 1 == 1);
                apsr.set_z(result != 0);
                apsr.set_c(false); //TODO carry
            }
            Opcode::UDF => unimplemented!(), //TODO UDF
            Opcode::UMAAL => {
                let dlo = inst.operands[0];
                let dhi = inst.operands[1];
                let n = self.read(inst.operands[2])? as u64;
                let m = self.read(inst.operands[3])? as u64;
                let result = n * m + self.read(dlo)? as u64 + self.read(dhi)? as u64;
                self.write(dlo, (result & 0xffffffff) as u32)?;
                self.write(dhi, (result >> 32) as u32)?;
            }
            Opcode::USAD8 | Opcode::USADA8 => {
                let d = inst.operands[0];
                let n = self.read(inst.operands[1])?.to_le_bytes();
                let m = self.read(inst.operands[2])?.to_le_bytes();
                let mut result = match inst.opcode {
                    Opcode::USAD8 => 0,
                    Opcode::USADA8 => self.read(inst.operands[3])?,
                    _ => unreachable!(),
                };
                for i in 0..4 {
                    result += (n[i] as i32 - m[i] as i32).abs() as u32;
                }
                self.write(d, result)?;
            }
            Opcode::WFE => unimplemented!(),   //TODO WFE
            Opcode::WFI => unimplemented!(),   //TODO WFI
            Opcode::YIELD => unimplemented!(), //TODO YIELD
        }
        Ok(())
    }
}
