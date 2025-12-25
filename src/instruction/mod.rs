pub mod arm;
pub mod thumb;
pub mod thumb2;

use crate::arithmetic::*;
use crate::machine::Machine;
use bitvec::prelude::*;

#[derive(Debug)]
pub enum ParseError {
    NotThumb,
    NotThumb2,
    NotArm,
}

//P174
#[derive(Debug)]
pub enum Operand {
    Immediate { value: u32 },                        //立即寻址(立即数)
    Register { reg_index: usize },                   //寄存器寻址
    RegWithShift { reg_index: usize, shift: Shift }, //寄存器移位寻址
}

#[repr(u8)]
#[derive(Debug)]
pub enum Condition {
    //P286
    EQ,
    NE,
    CS,
    CC,
    MI,
    PL,
    VS,
    VC,
    HI,
    LS,
    GE,
    LT,
    GT,
    LE,
    AL,
}

#[derive(Debug)]
pub struct Instruction {
    pub cond: Condition,
    pub setflags: bool,
    pub kind: InstructionKind,
    pub dest: Operand,
    pub operand1: Operand,
    pub operand2: Option<Operand>,
}

#[derive(Debug)]
pub enum InstructionKind {
    ADC, //加法并算上进位
    ADD, //加法
}

impl Condition {
    pub fn parse(cond: u8) -> Condition {
        match cond {
            0b0000 => Condition::EQ,
            0b0001 => Condition::NE,
            0b0010 => Condition::CS,
            0b0011 => Condition::CC,
            0b0100 => Condition::MI,
            0b0101 => Condition::PL,
            0b0110 => Condition::VS,
            0b0111 => Condition::VC,
            0b1000 => Condition::HI,
            0b1001 => Condition::LS,
            0b1010 => Condition::GE,
            0b1011 => Condition::LT,
            0b1100 => Condition::GT,
            0b1101 => Condition::LE,
            _ => Condition::AL,
        }
    }
}

impl Machine {
    //P230
    pub fn thumb_expand_imm(&self, imm12: u16) -> u32 {
        self.thumb_expand_imm_c(imm12, self.cpu.apsr().c()).0
    }

    //P231
    pub fn thumb_expand_imm_c(&self, imm12: u16, carry_in: bool) -> (u32, bool) {
        let imm32: u32;
        let carry_out;
        let imm12 = imm12.view_bits::<Lsb0>();
        let imm0_7 = imm12.get(0..8).unwrap().load::<u32>();
        if imm12.get(10..12).unwrap().load::<u32>() == 0b00 {
            match imm12.get(8..10).unwrap().load() {
                0b00 => imm32 = imm0_7,
                //imm32 = '00000000' : imm12<7:0> : '00000000' : imm12<7:0>;
                0b01 => imm32 = (imm0_7 << 16) | imm0_7,
                // imm32 = imm12<7:0> : '00000000' : imm12<7:0> : '00000000';
                0b10 => imm32 = (imm0_7 << 24) | (imm0_7 << 8),
                // imm32 = imm12<7:0> : imm12<7:0> : imm12<7:0> : imm12<7:0>;
                0b11 => imm32 = (imm0_7 << 24) | (imm0_7 << 16) | (imm0_7 << 8) | imm0_7,
                _ => unreachable!(),
            }
            carry_out = carry_in;
        } else {
            let unrotated_value = (1 << 7) | imm12.get(0..7).unwrap().load::<u32>();
            (imm32, carry_out) =
                self.rotate_right_c(unrotated_value, imm12.get(7..12).unwrap().load());
        }
        (imm32, carry_out)
    }

    //P199
    pub fn arm_expand_imm(&self, imm12: u16) -> u32 {
        self.arm_expand_imm_c(imm12, self.cpu.apsr().c()).0
    }

    //P199
    pub fn arm_expand_imm_c(&self, imm12: u16, carry_in: bool) -> (u32, bool) {
        let imm12 = imm12.view_bits::<Lsb0>();
        let unrotated_value = imm12.get(0..8).unwrap().load();
        self.shift_c(
            unrotated_value,
            Shift::RotateRight(2 * imm12.get(8..12).unwrap().load::<u32>()),
            carry_in,
        )
    }
}
