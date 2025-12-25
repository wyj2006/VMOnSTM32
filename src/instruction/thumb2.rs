use super::{Condition, Instruction, InstructionKind, Operand, ParseError};
use crate::{arithmetic::Shift, machine::Machine};
use bitvec::prelude::*;

impl Machine {
    ///解析32位thumb-2指令
    pub fn parse_thumb2(&mut self, instruction: u32) -> Result<Instruction, ParseError> {
        let instruction = instruction.view_bits::<Lsb0>();
        if instruction.get(27..32).unwrap().load::<u32>() == 0b111110
            && instruction.get(21..26).unwrap().load::<u32>() == 0b01010
            && instruction[15] == false
        {
            /* P298 Encoding T1
            ADC{S}<c> <Rd>, <Rn>, #<const>
            15 14 13 12 11 10 9 8 7 6 5 4 3 2 1 0 15 14 13 12 11 10 9 8 7 6 5 4 3 2 1 0
             1  1  1  1  0  i 0 1 0 1 0 S|  Rn   | 0| imm3   |  Rd     |     imm8
            d = UInt(Rd);  n = UInt(Rn);  setflags = (S == ‘1’);  imm32 = ThumbExpandImm(i:imm3:imm8);
            if d IN {13,15} || n IN {13,15} then UNPREDICTABLE;
            */
            return Ok(Instruction {
                cond: Condition::AL,
                setflags: instruction[20],
                kind: InstructionKind::ADC,
                dest: Operand::Register {
                    reg_index: instruction.get(8..12).unwrap().load(),
                },
                operand1: Operand::Register {
                    reg_index: instruction.get(16..20).unwrap().load(),
                },
                operand2: Some(Operand::Immediate {
                    value: self.thumb_expand_imm(
                        (instruction[26] as u16) << 11
                            | instruction.get(12..15).unwrap().load::<u16>() << 8
                            | (instruction.get(0..8).unwrap().load::<u16>()),
                    ),
                }),
            });
        }
        if instruction.get(5..16).unwrap().load::<u32>() == 0b11101011010
            && instruction[15] == false
        {
            /* P300 Encoding T2
            ADC{S}<c>.W <Rd>, <Rn>, <Rm>{, <shift>}
            15 14 13 12 11 10 9 8 7 6 5 4 3 2 1 0  15 14 13 12 11 10 9 8 7  6 5  4  3 2 1 0
             1  1  1  0  1  0 1 1 0 1 0 S|  Rn   |(0)|  imm3  |    Rd   |imm2|type|   Rm
            d = UInt(Rd);  n = UInt(Rn);  m = UInt(Rm);  setflags = (S == ‘1’);
            (shift_t, shift_n) = DecodeImmShift(type, imm3:imm2);
            if d IN {13,15} || n IN {13,15} || m IN {13,15} then UNPREDICTABLE;
            */
            return Ok(Instruction {
                cond: Condition::AL,
                setflags: instruction[20],
                kind: InstructionKind::ADC,
                dest: Operand::Register {
                    reg_index: instruction.get(8..12).unwrap().load(),
                },
                operand1: Operand::Register {
                    reg_index: instruction.get(16..20).unwrap().load(),
                },
                operand2: Some(Operand::RegWithShift {
                    reg_index: instruction.get(0..4).unwrap().load(),
                    shift: Shift::decode(
                        instruction.get(4..6).unwrap().load(),
                        instruction.get(12..15).unwrap().load::<u32>() << 2
                            | instruction.get(6..8).unwrap().load::<u32>(),
                    ),
                }),
            });
        }
        if instruction.get(27..32).unwrap().load::<u32>() == 0b11110
            && instruction.get(21..26).unwrap().load::<u32>() == 0b01000
            && instruction[15] == false
        {
            /* P304 Encoding T3
            ADD{S}<c>.W <Rd>, <Rn>, #<const>
            15 14 13 12 11 10 9 8 7 6 5 4 3 2 1 0 15 14 13 12 11 10 9 8 7 6 5 4 3 2 1 0
             1  1  1  1  0  i 0 1 0 0 0 S|  Rn   | 0|   imm3 |    Rd   |      imm8
            if Rd == ‘1111’ && S == ‘1’ then SEE CMN (immediate);
            if Rn == ‘1101’ then SEE ADD (SP plus immediate);
            d = UInt(Rd);  n = UInt(Rn);  setflags = (S == ‘1’);  imm32 = ThumbExpandImm(i:imm3:imm8);
            if d == 13 || (d == 15 && S == ‘0’) || n == 15 then UNPREDICTABLE;
            */
            let d = instruction.get(8..12).unwrap().load();
            let n = instruction.get(16..20).unwrap().load();
            let s = instruction[20];
            if !(d == 0b1111 && s == true || n == 0b1101) {
                return Ok(Instruction {
                    cond: Condition::AL,
                    setflags: s,
                    kind: InstructionKind::ADD,
                    dest: Operand::Register { reg_index: d },
                    operand1: Operand::Register { reg_index: n },
                    operand2: Some(Operand::Immediate {
                        value: self.thumb_expand_imm(
                            (instruction[26] as u16) << 11
                                | instruction.get(12..15).unwrap().load::<u16>() << 8
                                | (instruction.get(0..8).unwrap().load::<u16>()),
                        ),
                    }),
                });
            }
        }
        if instruction.get(27..32).unwrap().load::<u32>() == 0b11110
            && instruction.get(20..26).unwrap().load::<u32>() == 0b100000
            && instruction[15] == false
        {
            /* P304 Encoding T4
            ADDW<c> <Rd>, <Rn>, #<imm12>
            15 14 13 12 11 10 9 8 7 6 5 4 3 2 1 0 15 14 13 12 11 10 9 8 7 6 5 4 3 2 1 0
             1  1  1  1  0  i 1 0 0 0 0 0|  Rn   | 0|   imm3 |    Rd   |      imm8
            if Rn == ‘1111’ then SEE ADR;
            if Rn == ‘1101’ then SEE ADD (SP plus immediate);
            d = UInt(Rd);  n = UInt(Rn);  setflags = FALSE;  imm32 = ZeroExtend(i:imm3:imm8, 32);
            if d IN {13,15} then UNPREDICTABLE;
            */
            let n = instruction.get(16..20).unwrap().load();
            if !(n == 0b1111 || n == 0b1101) {
                return Ok(Instruction {
                    cond: Condition::AL,
                    setflags: false,
                    kind: InstructionKind::ADD,
                    dest: Operand::Register {
                        reg_index: instruction.get(8..12).unwrap().load(),
                    },
                    operand1: Operand::Register { reg_index: n },
                    operand2: Some(Operand::Immediate {
                        value: (instruction[26] as u32) << 11
                            | instruction.get(12..15).unwrap().load::<u32>() << 8
                            | (instruction.get(0..8).unwrap().load::<u32>()),
                    }),
                });
            }
        }
        if instruction.get(21..32).unwrap().load::<u32>() == 0b11101011000
            && instruction[15] == false
        {
            /* P308 Encoding T3
            ADD{S}<c>.W <Rd>, <Rn>, <Rm>{, <shift>}
            15 14 13 12 11 10 9 8 7 6 5 4 3 2 1 0  15 14 13 12 11 10 9 8 7  6 5  4 3 2 1 0
             1  1  1  0  1  0 1 1 0 0 0 S|  Rn   |(0)| imm3   |    Rd   |imm2|type| Rm
            if Rd == ‘1111’ && S == ‘1’ then SEE CMN (register);
            if Rn == ‘1101’ then SEE ADD (SP plus register);
            d = UInt(Rd);  n = UInt(Rn);  m = UInt(Rm);  setflags = (S == ‘1’);
            (shift_t, shift_n) = DecodeImmShift(type, imm3:imm2);
            if d == 13 || (d == 15 && S == ‘0’) || n == 15 || m IN {13,15} then UNPREDICTABLE;
            */
            let d = instruction.get(8..12).unwrap().load();
            let n = instruction.get(16..20).unwrap().load();
            let s = instruction[20];
            if !(d == 0b1111 && s == true || n == 0b1101) {
                return Ok(Instruction {
                    cond: Condition::AL,
                    setflags: s,
                    kind: InstructionKind::ADD,
                    dest: Operand::Register { reg_index: d },
                    operand1: Operand::Register { reg_index: n },
                    operand2: Some(Operand::RegWithShift {
                        reg_index: instruction.get(0..4).unwrap().load(),
                        shift: Shift::decode(
                            instruction.get(4..6).unwrap().load(),
                            instruction.get(12..15).unwrap().load::<u32>() << 2
                                | instruction.get(6..8).unwrap().load::<u32>(),
                        ),
                    }),
                });
            }
        }
        if instruction.get(27..32).unwrap().load::<u32>() == 0b11110
            && instruction.get(21..26).unwrap().load::<u32>() == 0b01000
            && instruction.get(15..20).unwrap().load::<u32>() == 0b11010
        {
            /* P314 Encoding T3
            ADD{S}<c>.W <Rd>, SP, #<const>
            15 14 13 12 11 10 9 8 7 6 5 4 3 2 1 0 15 14 13 12 11 10 9 8 7 6 5 4 3 2 1 0
             1  1  1  1  0  i 0 1 0 0 0 S 1 1 0 1  0|  imm3  |    Rd   |  imm8
            if Rd == ‘1111’ && S == ‘1’ then SEE CMN (immediate);
            d = UInt(Rd);  setflags = (S == ‘1’);  imm32 = ThumbExpandImm(i:imm3:imm8);
            if d == 15 && S == ‘0’ then UNPREDICTABLE;
            */
            let d = instruction.get(8..12).unwrap().load();
            let s = instruction[20];
            if !(d == 0b1111 && s == true) {
                return Ok(Instruction {
                    cond: Condition::AL,
                    setflags: s,
                    kind: InstructionKind::ADD,
                    dest: Operand::Register { reg_index: d },
                    operand1: Operand::Register {
                        reg_index: self.cpu.sp_index,
                    },
                    operand2: Some(Operand::Immediate {
                        value: self.thumb_expand_imm(
                            (instruction[26] as u16) << 11
                                | instruction.get(12..15).unwrap().load::<u16>() << 8
                                | instruction.get(0..8).unwrap().load::<u16>(),
                        ),
                    }),
                });
            }
        }
        if instruction.get(27..32).unwrap().load::<u32>() == 0b11110
            && instruction.get(15..26).unwrap().load::<u32>() == 0b10000011010
        {
            /* P314 Encoding T4
            ADDW<c> <Rd>, SP, #<imm12>
            15 14 13 12 11 10 9 8 7 6 5 4 3 2 1 0 15 14 13 12 11 10 9 8 7 6 5 4 3 2 1 0
             1  1  1  1  0  i 1 0 0 0 0 0 1 1 0 1  0|  imm3  |    Rd   |      imm8
            d = UInt(Rd);  setflags = FALSE;  imm32 = ZeroExtend(i:imm3:imm8, 32);
            if d == 15 then UNPREDICTABLE;
            */
            return Ok(Instruction {
                cond: Condition::AL,
                setflags: false,
                kind: InstructionKind::ADD,
                dest: Operand::Register {
                    reg_index: instruction.get(8..12).unwrap().load(),
                },
                operand1: Operand::Register {
                    reg_index: self.cpu.sp_index,
                },
                operand2: Some(Operand::Immediate {
                    value: (instruction[26] as u32) << 11
                        | instruction.get(12..15).unwrap().load::<u32>() << 8
                        | instruction.get(0..8).unwrap().load::<u32>(),
                }),
            });
        }
        if instruction.get(21..32).unwrap().load::<u32>() == 0b11101011000
            && instruction.get(15..20).unwrap().load::<u32>() == 0b11010
        {
            /* P316 Encoding T3
            ADD{S}<c>.W <Rd>, SP, <Rm>{, <shift>}
            15 14 13 12 11 10 9 8 7 6 5 4 3 2 1 0  15 14 13 12 11 10 9 8 7  6 5  4 3 2 1 0
             1  1  1  0  1  0 1 1 0 0 0 S 1 1 0 1 (0)|  imm3  |    Rd   |imm2|type|   Rm
            if Rd == ‘1111’ && S == ‘1’ then SEE CMN (register);
            d = UInt(Rd);  m = UInt(Rm);  setflags = (S == ‘1’);
            (shift_t, shift_n) = DecodeImmShift(type, imm3:imm2);
            if d == 13 && (shift_t != SRType_LSL || shift_n > 3) then UNPREDICTABLE;
            if (d == 15 && S == ‘0’) || m IN {13,15} then UNPREDICTABLE;
            */
            let d = instruction.get(8..12).unwrap().load();
            let s = instruction[20];
            if !(d == 0b1111 && s == true) {
                return Ok(Instruction {
                    cond: Condition::AL,
                    setflags: s,
                    kind: InstructionKind::ADD,
                    dest: Operand::Register { reg_index: d },
                    operand1: Operand::Register {
                        reg_index: self.cpu.sp_index,
                    },
                    operand2: Some(Operand::RegWithShift {
                        reg_index: instruction.get(0..4).unwrap().load(),
                        shift: Shift::decode(
                            instruction.get(4..6).unwrap().load(),
                            instruction.get(12..15).unwrap().load::<u32>() << 2
                                | instruction.get(6..8).unwrap().load::<u32>(),
                        ),
                    }),
                });
            }
        }
        Err(ParseError::NotThumb2)
    }
}
