use super::{Condition, Instruction, InstructionKind, Operand, ParseError};
use crate::{arithmetic::Shift, machine::Machine};
use bitvec::prelude::*;

impl Machine {
    ///解析arm指令
    pub fn parse_arm(&self, instruction: u32) -> Result<Instruction, ParseError> {
        let instruction = instruction.view_bits::<Lsb0>();
        if instruction.get(21..28).unwrap().load::<u32>() == 0b0010101 {
            /* P298 Encoding A1
            ADC{S}<c> <Rd>, <Rn>, #<const>
            31 30 29 28 27 26 25 24 23 22 21 20 19 18 17 16 15 14 13 12 11 10 9 8 7 6 5 4 3 2 1 0
               cond    | 0  0  1  0  1  0  1  S|    Rn     |     Rd    |          imm12
            For the case when cond is 0b1111, see Unconditional instructions on page A5-214.
            if Rd == ‘1111’ && S == ‘1’ then SEE SUBS PC, LR and related instructions;
            d = UInt(Rd);  n = UInt(Rn);  setflags = (S == ‘1’);  imm32 = ARMExpandImm(imm12);
            */
            let d = instruction.get(12..16).unwrap().load();
            let s = instruction[20];
            if !(d == 0b1111 && s == true) {
                return Ok(Instruction {
                    cond: Condition::parse(instruction.get(28..32).unwrap().load()),
                    setflags: s,
                    kind: InstructionKind::ADC,
                    dest: Operand::Register { reg_index: d },
                    operand1: Operand::Register {
                        reg_index: instruction.get(16..20).unwrap().load(),
                    },
                    operand2: Some(Operand::Immediate {
                        value: self.arm_expand_imm(instruction.get(0..12).unwrap().load()),
                    }),
                });
            }
        }
        if instruction.get(21..28).unwrap().load::<u32>() == 0b0000101 && instruction[4] == false {
            /* P300 Encoding A1
            ADC{S}<c> <Rd>, <Rn>, <Rm>{, <shift>}
            31 30 29 28 27 26 25 24 23 22 21 20 19 18 17 16 15 14 13 12 11 10 9 8 7 6  5 4 3 2 1 0
                cond   | 0  0  0  0  1  0  1  S|    Rn     |    Rd     |    imm5   |type|0|   Rm
            For the case when cond is 0b1111, see Unconditional instructions on page A5-214.
            if Rd == ‘1111’ && S == ‘1’ then SEE SUBS PC, LR and related instructions;
            d = UInt(Rd);  n = UInt(Rn);  m = UInt(Rm);  setflags = (S == ‘1’);
            (shift_t, shift_n) = DecodeImmShift(type, imm5);
            */
            let d = instruction.get(12..16).unwrap().load();
            let s = instruction[20];
            if !(d == 0b1111 && s == true) {
                return Ok(Instruction {
                    cond: Condition::parse(instruction.get(28..32).unwrap().load()),
                    setflags: s,
                    kind: InstructionKind::ADC,
                    dest: Operand::Register { reg_index: d },
                    operand1: Operand::Register {
                        reg_index: instruction.get(16..20).unwrap().load(),
                    },
                    operand2: Some(Operand::RegWithShift {
                        reg_index: instruction.get(0..4).unwrap().load(),
                        shift: Shift::decode(
                            instruction.get(5..7).unwrap().load(),
                            instruction.get(7..12).unwrap().load(),
                        ),
                    }),
                });
            }
        }
        if instruction.get(21..28).unwrap().load::<u32>() == 0b0000101
            && instruction[7] == false
            && instruction[4] == true
        {
            /* P302 Encoding A1
            ADC{S}<c> <Rd>, <Rn>, <Rm>, <type> <Rs>
            31 30 29 28 27 26 25 24 23 22 21 20 19 18 17 16 15 14 13 12 11 10 9 8 7 6  5 4 3 2 1 0
                cond   | 0  0  0  0  1  0  1  S|     Rn    |     Rd    |    Rs   |0|type|1|  Rm
            For the case when cond is 0b1111, see Unconditional instructions on page A5-214.
            d = UInt(Rd);  n = UInt(Rn);  m = UInt(Rm);  s = UInt(Rs);
            setflags = (S == ‘1’);  shift_t = DecodeRegShift(type);
            if d == 15 || n == 15 || m == 15 || s == 15 then UNPREDICTABLE;
            */
            return Ok(Instruction {
                cond: Condition::parse(instruction.get(28..32).unwrap().load()),
                setflags: instruction[20],
                kind: InstructionKind::ADC,
                dest: Operand::Register {
                    reg_index: instruction.get(12..16).unwrap().load(),
                },
                operand1: Operand::Register {
                    reg_index: instruction.get(16..20).unwrap().load(),
                },
                operand2: Some(Operand::RegWithShift {
                    reg_index: instruction.get(0..4).unwrap().load(),
                    shift: Shift::decode_reg(
                        instruction.get(5..7).unwrap().load(),
                        instruction.get(8..12).unwrap().load(),
                    ),
                }),
            });
        }
        if instruction.get(21..28).unwrap().load::<u32>() == 0b0010100 {
            /*P306 Encoding A1
            ADD{S}<c> <Rd>, <Rn>, #<const>
            31 30 29 28 27 26 25 24 23 22 21 20 19 18 17 16 15 14 13 12 11 10 9 8 7 6 5 4 3 2 1 0
               cond    | 0  0  1  0  1  0  0  S|    Rn     |    Rd     |           imm12
            For the case when cond is 0b1111, see Unconditional instructions on page A5-214.
            if Rn == ‘1111’ && S == ‘0’ then SEE ADR;
            if Rn == ‘1101’ then SEE ADD (SP plus immediate);
            if Rd == ‘1111’ && S == ‘1’ then SEE SUBS PC, LR and related instructions;
            d = UInt(Rd);  n = UInt(Rn);  setflags = (S == ‘1’);  imm32 = ARMExpandImm(imm12);
            */
            let d = instruction.get(12..16).unwrap().load();
            let n = instruction.get(16..20).unwrap().load();
            let s = instruction[20];
            if !(n == 0b1111 && s == false || n == 0b1101 || d == 0b1111 && s == true) {
                return Ok(Instruction {
                    cond: Condition::parse(instruction.get(28..32).unwrap().load()),
                    setflags: s,
                    kind: InstructionKind::ADD,
                    dest: Operand::Register { reg_index: d },
                    operand1: Operand::Register { reg_index: n },
                    operand2: Some(Operand::Immediate {
                        value: self.arm_expand_imm(instruction.get(0..12).unwrap().load()),
                    }),
                });
            }
        }
        if instruction.get(21..28).unwrap().load::<u32>() == 0b0000100 && instruction[4] == false {
            /* P310 Encoding A1
            ADD{S}<c> <Rd>, <Rn>, <Rm>{, <shift>}
            31 30 29 28 27 26 25 24 23 22 21 20 19 18 17 16 15 14 13 12 11 10 9 8 7 6  5 4 3 2 1 0
               cond    | 0  0  0  0  1  0  0  S|    Rn     |    Rd     |    imm5   |type|0|  Rm
            For the case when cond is 0b1111, see Unconditional instructions on page A5-214.
            if Rd == ‘1111’ && S == ‘1’ then SEE SUBS PC, LR and related instructions;
            if Rn == ‘1101’ then SEE ADD (SP plus register);
            d = UInt(Rd);  n = UInt(Rn);  m = UInt(Rm);  setflags = (S == ‘1’);
            (shift_t, shift_n) = DecodeImmShift(type, imm5);
            */
            let d = instruction.get(12..16).unwrap().load();
            let n = instruction.get(16..20).unwrap().load();
            let s = instruction[20];
            if !(d == 0b1111 && s == true || n == 0b1111) {
                return Ok(Instruction {
                    cond: Condition::parse(instruction.get(28..32).unwrap().load()),
                    setflags: s,
                    kind: InstructionKind::ADD,
                    dest: Operand::Register { reg_index: d },
                    operand1: Operand::Register { reg_index: n },
                    operand2: Some(Operand::RegWithShift {
                        reg_index: instruction.get(0..4).unwrap().load(),
                        shift: Shift::decode(
                            instruction.get(5..7).unwrap().load(),
                            instruction.get(7..12).unwrap().load(),
                        ),
                    }),
                });
            }
        }
        if instruction.get(21..28).unwrap().load::<u32>() == 0b0000100
            && instruction[7] == false
            && instruction[4] == true
        {
            /* P312 Encoding A1
            ADD{S}<c> <Rd>, <Rn>, <Rm>, <type> <Rs>
            31 30 29 28 27 26 25 24 23 22 21 20 19 18 17 16 15 14 13 12 11 10 9 8 7 6  5 4 3 2 1 0
               cond    | 0  0  0  0  1  0  0  S|     Rn    |    Rd     |    Rs   |0|type|1|   Rm
            For the case when cond is 0b1111, see Unconditional instructions on page A5-214.
            d = UInt(Rd);  n = UInt(Rn);  m = UInt(Rm);  s = UInt(Rs);
            setflags = (S == ‘1’);  shift_t = DecodeRegShift(type);
            if d == 15 || n == 15 || m == 15 || s == 15 then UNPREDICTABLE;
            */
            return Ok(Instruction {
                cond: Condition::parse(instruction.get(28..32).unwrap().load()),
                setflags: instruction[20],
                kind: InstructionKind::ADD,
                dest: Operand::Register {
                    reg_index: instruction.get(12..16).unwrap().load(),
                },
                operand1: Operand::Register {
                    reg_index: instruction.get(16..20).unwrap().load(),
                },
                operand2: Some(Operand::RegWithShift {
                    reg_index: instruction.get(0..4).unwrap().load(),
                    shift: Shift::decode_reg(
                        instruction.get(5..7).unwrap().load(),
                        instruction.get(8..12).unwrap().load(),
                    ),
                }),
            });
        }
        if instruction.get(21..28).unwrap().load::<u32>() == 0b0010100
            && instruction.get(16..20).unwrap().load::<u32>() == 0b1101
        {
            /* P314 Encoding A1
            ADD{S}<c> <Rd>, SP, #<const>
            31 30 29 28 27 26 25 24 23 22 21 20 19 18 17 16 15 14 13 12 11 10 9 8 7 6 5 4 3 2 1 0
               cond    | 0  0  1  0  1  0  0  S  1  1  0  1|    Rd     |           imm12
            For the case when cond is 0b1111, see Unconditional instructions on page A5-214.
            if Rd == ‘1111’ && S == ‘1’ then SEE SUBS PC, LR and related instructions;
            d = UInt(Rd);  setflags = (S == ‘1’);  imm32 = ARMExpandImm(imm12);
            */
            let d = instruction.get(12..16).unwrap().load();
            let s = instruction[20];
            if !(d == 0b1111 && s == true) {
                return Ok(Instruction {
                    cond: Condition::parse(instruction.get(28..32).unwrap().load()),
                    setflags: s,
                    kind: InstructionKind::ADD,
                    dest: Operand::Register { reg_index: d },
                    operand1: Operand::Register {
                        reg_index: self.cpu.sp_index,
                    },
                    operand2: Some(Operand::Immediate {
                        value: self.arm_expand_imm(instruction.get(0..12).unwrap().load()),
                    }),
                });
            }
        }
        if instruction.get(21..28).unwrap().load::<u32>() == 0b0000100
            && instruction.get(16..20).unwrap().load::<u32>() == 0b1101
            && instruction[4] == false
        {
            /* P318 Encoding A1
            ADD{S}<c> <Rd>, SP, <Rm>{, <shift>}
            31 30 29 28 27 26 25 24 23 22 21 20 19 18 17 16 15 14 13 12 11 10 9 8 7 6  5 4 3 2 1 0
               cond    | 0  0  0  0  1  0  0  S  1  1  0  1|    Rd     |    imm5   |type|0|  Rm
            For the case when cond is 0b1111, see Unconditional instructions on page A5-214.
            if Rd == ‘1111’ && S == ‘1’ then SEE SUBS PC, LR and related instructions;
            d = UInt(Rd);  m = UInt(Rm);  setflags = (S == ‘1’);
            (shift_t, shift_n) = DecodeImmShift(type, imm5);
            */
            let d = instruction.get(12..16).unwrap().load();
            let s = instruction[20];
            if !(d == 0b1111 && s == true) {
                return Ok(Instruction {
                    cond: Condition::parse(instruction.get(28..32).unwrap().load()),
                    setflags: s,
                    kind: InstructionKind::ADD,
                    dest: Operand::Register { reg_index: d },
                    operand1: Operand::Register {
                        reg_index: self.cpu.sp_index,
                    },
                    operand2: Some(Operand::RegWithShift {
                        reg_index: instruction.get(0..4).unwrap().load(),
                        shift: Shift::decode(
                            instruction.get(5..7).unwrap().load(),
                            instruction.get(7..12).unwrap().load(),
                        ),
                    }),
                });
            }
        }
        Err(ParseError::NotArm)
    }
}
