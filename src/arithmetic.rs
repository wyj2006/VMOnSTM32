use crate::machine::Machine;

//P289
#[derive(Debug)]
pub enum Shift {
    LogicLeft(u32),
    LogicRight(u32),
    ArithRight(u32),
    RotateRight(u32),
    RotateRightExtend,
    RegLogicLeft(usize),
    RegLogicRight(usize),
    RegArithRight(usize),
    RegRotateRight(usize),
}

impl Shift {
    //P290
    pub fn decode(r#type: u8, imm5: u32) -> Shift {
        match r#type {
            0b00 => Shift::LogicLeft(imm5),
            0b01 => Shift::LogicRight(if imm5 & 0b11111 == 0b00000 { 32 } else { imm5 }),
            0b10 => Shift::ArithRight(if imm5 & 0b11111 == 0b00000 { 32 } else { imm5 }),
            0b11 => {
                if imm5 & 0b11111 == 0b0000 {
                    Shift::RotateRightExtend
                } else {
                    Shift::RotateRight(imm5)
                }
            }
            _ => unreachable!(),
        }
    }

    //290
    pub fn decode_reg(r#type: u8, reg_index: usize) -> Shift {
        match r#type {
            0b00 => Shift::RegLogicLeft(reg_index),
            0b01 => Shift::RegLogicRight(reg_index),
            0b10 => Shift::RegArithRight(reg_index),
            _ => unreachable!(),
        }
    }
}

impl Default for Shift {
    fn default() -> Self {
        Shift::LogicLeft(0)
    }
}

impl Machine {
    //P41
    pub fn logic_left_c(&self, value: u32, shift: u32) -> (u32, bool) {
        if shift == 0 {
            (value, false)
        } else {
            (value << shift, value << (shift - 1) >> 31 & 1 == 1)
        }
    }

    //P42
    pub fn logic_left(&self, value: u32, shift: u32) -> u32 {
        self.logic_left_c(value, shift).0
    }

    //P42
    pub fn logic_right_c(&self, value: u32, shift: u32) -> (u32, bool) {
        if shift == 0 {
            (value, false)
        } else {
            (value >> shift, value >> (shift - 1) & 1 == 1)
        }
    }

    //P42
    pub fn logic_right(&self, value: u32, shift: u32) -> u32 {
        self.logic_right_c(value, shift).0
    }

    //P42
    pub fn arith_right_c(&self, value: u32, shift: u32) -> (u32, bool) {
        if shift == 0 {
            (value, false)
        } else {
            (
                ((value as i32) >> shift) as u32,
                value >> (shift - 1) & 1 == 1,
            )
        }
    }

    //P42
    pub fn arith_right(&self, value: u32, shift: u32) -> u32 {
        self.arith_right_c(value, shift).0
    }

    //P42
    pub fn rotate_right_c(&self, value: u32, shift: u32) -> (u32, bool) {
        if shift == 0 {
            (value, false)
        } else {
            let shift = shift % 32;
            let result = (value >> shift) | (value << (32 - shift));
            let carry_out = result >> 31 & 1 == 1;
            (result, carry_out)
        }
    }

    //P43
    pub fn rotate_right(&self, value: u32, shift: u32) -> u32 {
        self.rotate_right_c(value, shift).0
    }

    //P43
    pub fn rotate_right_extend_c(&self, value: u32, carry_in: bool) -> (u32, bool) {
        ((carry_in as u32) << 31 | value >> 1, value & 1 == 1)
    }

    //P43
    pub fn rotate_right_extend(&self, value: u32, carry_in: bool) -> u32 {
        self.rotate_right_extend_c(value, carry_in).0
    }

    //P290
    pub fn shift_c(&self, value: u32, shift: Shift, carry_in: bool) -> (u32, bool) {
        match shift {
            Shift::LogicLeft(amount) => self.logic_left_c(value, amount),
            Shift::LogicRight(amount) => self.logic_right_c(value, amount),
            Shift::ArithRight(amount) => self.arith_right_c(value, amount),
            Shift::RotateRight(amount) => self.rotate_right_c(value, amount),
            Shift::RotateRightExtend => self.rotate_right_extend_c(value, carry_in),
            Shift::RegLogicLeft(reg_index) => self.logic_left_c(value, self.cpu.regs[reg_index]),
            Shift::RegLogicRight(reg_index) => self.logic_right_c(value, self.cpu.regs[reg_index]),
            Shift::RegArithRight(reg_index) => self.arith_right_c(value, self.cpu.regs[reg_index]),
            Shift::RegRotateRight(reg_index) => {
                self.rotate_right_c(value, self.cpu.regs[reg_index])
            }
        }
    }

    //P290
    pub fn shift(&self, value: u32, shift: Shift, carry_in: bool) -> u32 {
        self.shift_c(value, shift, carry_in).0
    }

    //P43
    pub fn add_with_carry(&self, x: u32, y: u32, carry_in: bool) -> (u32, bool, bool) {
        let unsigned_sum = x + y + (carry_in as u32);
        let signed_num = (x as i32) + (y as i32) + (carry_in as i32);
        let result = unsigned_sum & !(1 << (u32::BITS - 1)); //保留后31位
        let carry_out = result != unsigned_sum;
        let overflow = (result as i32) != signed_num;
        (result, carry_out, overflow)
    }
}
