use super::{Condition, Instruction, InstructionKind, Operand, ParseError};
use crate::arithmetic::*;
use crate::machine::Machine;
use bitvec::prelude::*;

impl Machine {
    ///解析16位的thumb-2指令
    pub fn parse_thumb(&self, instruction: u16) -> Result<Instruction, ParseError> {
        let instruction = instruction.view_bits::<Lsb0>();
        if instruction.get(6..16).unwrap().load::<u32>() == 0b0100000101 {
            /* P300 Encoding T1
            ADCS <Rdn>, <Rm>      Outside IT block.
            ADC<c> <Rdn>, <Rm>    Inside IT block.
            15 14 13 12 11 10 9 8 7 6 5 4 3 2 1 0
             0  1  0  0  0  0 0 1 0 1| Rm  | Rdn
            d = UInt(Rdn);  n = UInt(Rdn);  m = UInt(Rm);  setflags = !InITBlock();
            (shift_t, shift_n) = (SRType_LSL, 0);
            */
            return Ok(Instruction {
                cond: Condition::AL,
                setflags: !self.in_it_block(),
                kind: InstructionKind::ADC,
                dest: Operand::Register {
                    reg_index: instruction.get(0..3).unwrap().load(),
                },
                operand1: Operand::Register {
                    reg_index: instruction.get(0..3).unwrap().load(),
                },
                operand2: Some(Operand::RegWithShift {
                    reg_index: instruction.get(3..6).unwrap().load(),
                    shift: Shift::LogicLeft(0),
                }),
            });
        }
        if instruction.get(9..16).unwrap().load::<u32>() == 0b0001110 {
            /* P304 Encoding T1
            ADDS <Rd>,<Rn>,#<imm3>      Outside IT block.
            ADD<c> <Rd>,<Rn>,#<imm3>    Inside IT block.
            15 14 13 12 11 10 9 8 7 6 5 4 3 2 1 0
             0  0  0  1  1  1 0| imm3|  Rn |Rd
            d = UInt(Rd);  n = UInt(Rn);  setflags = !InITBlock();  imm32 = ZeroExtend(imm3, 32);
            */
            return Ok(Instruction {
                cond: Condition::AL,
                setflags: !self.in_it_block(),
                kind: InstructionKind::ADD,
                dest: Operand::Register {
                    reg_index: instruction.get(0..3).unwrap().load(),
                },
                operand1: Operand::Register {
                    reg_index: instruction.get(3..6).unwrap().load(),
                },
                operand2: Some(Operand::Immediate {
                    value: instruction.get(6..9).unwrap().load(),
                }),
            });
        }
        if instruction.get(11..16).unwrap().load::<u32>() == 0b00110 {
            /* P304 Encoding T2
            ADDS <Rdn>,#<imm8>      Outside IT block.
            ADD<c> <Rdn>,#<imm8>    Inside IT block.
            15 14 13 12 11 10 9 8 7 6 5 4 3 2 1 0
             0  0  1  1  0| Rdn  |     imm8
            d = UInt(Rdn);  n = UInt(Rdn);  setflags = !InITBlock();  imm32 = ZeroExtend(imm8, 32);
            */
            return Ok(Instruction {
                cond: Condition::AL,
                setflags: !self.in_it_block(),
                kind: InstructionKind::ADD,
                dest: Operand::Register {
                    reg_index: instruction.get(8..11).unwrap().load(),
                },
                operand1: Operand::Register {
                    reg_index: instruction.get(8..11).unwrap().load(),
                },
                operand2: Some(Operand::Immediate {
                    value: instruction.get(0..8).unwrap().load(),
                }),
            });
        }
        if instruction.get(9..16).unwrap().load::<u32>() == 0b0001100 {
            /* P308 Encoding T1
            ADDS <Rd>,<Rn>,<Rm>      Outside IT block.
            ADD<c> <Rd>,<Rn>,<Rm>    Inside IT block.
            15 14 13 12 11 10 9 8 7 6 5 4 3 2 1 0
             0  0  0  1  1  0 0| Rm  | Rn  | Rd
            d = UInt(Rd);  n = UInt(Rn);  m = UInt(Rm);  setflags = !InITBlock();
            (shift_t, shift_n) = (SRType_LSL, 0)
            */
            return Ok(Instruction {
                cond: Condition::AL,
                setflags: !self.in_it_block(),
                kind: InstructionKind::ADD,
                dest: Operand::Register {
                    reg_index: instruction.get(0..3).unwrap().load(),
                },
                operand1: Operand::Register {
                    reg_index: instruction.get(3..6).unwrap().load(),
                },
                operand2: Some(Operand::RegWithShift {
                    reg_index: instruction.get(6..9).unwrap().load(),
                    shift: Shift::LogicLeft(0),
                }),
            });
        }
        if instruction.get(9..16).unwrap().load::<u32>() == 0b01000100 {
            /* P308 Encoding T2
            ADD<c> <Rdn>,<Rm>           If <Rdn> is the PC, must be outside or last in IT block.
            15 14 13 12 11 10 9 8  7 6 5 4 3 2 1 0
             0  1  0  0  0  1 0 0 Dn|  Rm   | Rdn
            if (DN:Rdn) == '1101' || Rm == '1101' then SEE ADD (SP plus register);
            d = UInt(DN:Rdn);  n = UInt(DN:Rdn);  m = UInt(Rm);  setflags = FALSE;
            (shift_t, shift_n) = (SRType_LSL, 0);
            if d == 15 && InITBlock() && !LastInITBlock() then UNPREDICTABLE;
            if d == 15 && m == 15 then UNPREDICTABLE;
            */
            let d =
                ((instruction[7] as usize) << 3) | instruction.get(0..3).unwrap().load::<usize>();
            let m = instruction.get(3..7).unwrap().load();
            if !(d == 0b1101 || m == 0b1101) {
                return Ok(Instruction {
                    cond: Condition::AL,
                    setflags: false,
                    kind: InstructionKind::ADD,
                    dest: Operand::Register { reg_index: d },
                    operand1: Operand::Register { reg_index: d },
                    operand2: Some(Operand::RegWithShift {
                        reg_index: m,
                        shift: Shift::LogicLeft(0),
                    }),
                });
            }
        }
        if instruction.get(11..16).unwrap().load::<u32>() == 0b10101 {
            /* P314 Encoding T1
            ADD<c> <Rd>, SP, #<imm>
            15 14 13 12 11 10 9 8 7 6 5 4 3 2 1 0
             1  0  1  0  1|  Rd  |       imm8
            d = UInt(Rd);  setflags = FALSE;  imm32 = ZeroExtend(imm8:’00’, 32);
            */
            return Ok(Instruction {
                cond: Condition::AL,
                setflags: false,
                kind: InstructionKind::ADD,
                dest: Operand::Register {
                    reg_index: instruction.get(8..11).unwrap().load(),
                },
                operand1: Operand::Register {
                    reg_index: self.cpu.sp_index,
                },
                operand2: Some(Operand::Immediate {
                    value: instruction.get(0..8).unwrap().load::<u32>() << 2,
                }),
            });
        }
        if instruction.get(7..16).unwrap().load::<u32>() == 0b101100000 {
            /* P314 Encoding T2
            ADD<c> SP, SP, #<imm>
            15 14 13 12 11 10 9 8 7 6 5 4 3 2 1 0
             1  0  1  1  0  0 0 0 0|    imm7
            d = 13;  setflags = FALSE;  imm32 = ZeroExtend(imm7:’00’, 32);
            */
            return Ok(Instruction {
                cond: Condition::AL,
                setflags: false,
                kind: InstructionKind::ADD,
                dest: Operand::Register {
                    reg_index: self.cpu.sp_index,
                },
                operand1: Operand::Register {
                    reg_index: self.cpu.sp_index,
                },
                operand2: Some(Operand::Immediate {
                    value: instruction.get(0..7).unwrap().load::<u32>() << 2,
                }),
            });
        }
        if instruction.get(8..16).unwrap().load::<u32>() == 0b01000100
            && instruction.get(3..7).unwrap().load::<u32>() == 0b1101
        {
            /* P316 Encoding T1
            ADD<c> <Rdm>, SP, <Rdm>
            15 14 13 12 11 10 9 8  7 6 5 4 3 2 1 0
             0  1  0  0  0  1 0 0 DM 1 1 0 1| Rdm
            d = UInt(DM:Rdm);  m = UInt(DM:Rdm);  setflags = FALSE;
            if d == 15 && InITBlock() && !LastInITBlock() then UNPREDICTABLE;
            (shift_t, shift_n) = (SRType_LSL, 0);
            */
            let d = (instruction[7] as usize) << 3 | instruction.get(0..3).unwrap().load::<usize>();
            let m = d;
            return Ok(Instruction {
                cond: Condition::AL,
                setflags: false,
                kind: InstructionKind::ADD,
                dest: Operand::Register { reg_index: d },
                operand1: Operand::Register {
                    reg_index: self.cpu.sp_index,
                },
                operand2: Some(Operand::RegWithShift {
                    reg_index: m,
                    shift: Shift::LogicLeft(0),
                }),
            });
        }
        if instruction.get(7..16).unwrap().load::<u32>() == 0b010001001
            && instruction.get(0..3).unwrap().load::<u32>() == 0b101
        {
            /* P316 Encoding T2
            ADD<c> SP, <Rm>
            15 14 13 12 11 10 9 8 7 6 5 4 3 2 1 0
             0  1  0  0  0  1 0 0 1|   Rm  |1 0 1
            if Rm == ‘1101’ then SEE encoding T1;
            d = 13;  m = UInt(Rm);  setflags = FALSE;
            (shift_t, shift_n) = (SRType_LSL, 0);
            */
            let m: usize = instruction.get(3..7).unwrap().load();
            if !m == 0b1101 {
                return Ok(Instruction {
                    cond: Condition::AL,
                    setflags: false,
                    kind: InstructionKind::ADD,
                    dest: Operand::Register {
                        reg_index: self.cpu.sp_index,
                    },
                    operand1: Operand::Register {
                        reg_index: self.cpu.sp_index,
                    },
                    operand2: Some(Operand::RegWithShift {
                        reg_index: m,
                        shift: Shift::LogicLeft(0),
                    }),
                });
            }
        }
        Err(ParseError::NotThumb)
    }
}
