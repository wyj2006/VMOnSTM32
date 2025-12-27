use yaxpeax_arm::armv7::ShiftStyle;

//P41
pub fn logic_left_with_carry(value: u32, shift: u32) -> (u32, bool) {
    if shift == 0 {
        (value, false)
    } else {
        (value << shift, value << (shift - 1) >> 31 & 1 == 1)
    }
}

//P42
pub fn logic_left(value: u32, shift: u32) -> u32 {
    logic_left_with_carry(value, shift).0
}

//P42
pub fn logic_right_with_carry(value: u32, shift: u32) -> (u32, bool) {
    if shift == 0 {
        (value, false)
    } else {
        (value >> shift, value >> (shift - 1) & 1 == 1)
    }
}

//P42
pub fn logic_right(value: u32, shift: u32) -> u32 {
    logic_right_with_carry(value, shift).0
}

//P42
pub fn arith_right_with_carry(value: u32, shift: u32) -> (u32, bool) {
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
pub fn arith_right(value: u32, shift: u32) -> u32 {
    arith_right_with_carry(value, shift).0
}

//P42
pub fn rotate_right_with_carry(value: u32, shift: u32) -> (u32, bool) {
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
pub fn rotate_right(value: u32, shift: u32) -> u32 {
    rotate_right_with_carry(value, shift).0
}

//P43
pub fn rotate_right_extend_with_carry(value: u32, carry_in: bool) -> (u32, bool) {
    ((carry_in as u32) << 31 | value >> 1, value & 1 == 1)
}

//P43
pub fn rotate_right_extend(value: u32, carry_in: bool) -> u32 {
    rotate_right_extend_with_carry(value, carry_in).0
}

//P290
pub fn shift_c(value: u32, shift_style: ShiftStyle, amount: u32, carry_in: bool) -> (u32, bool) {
    match shift_style {
        ShiftStyle::LSL => logic_left_with_carry(value, amount),
        ShiftStyle::LSR => logic_right_with_carry(value, amount),
        ShiftStyle::ASR => arith_right_with_carry(value, amount),
        ShiftStyle::ROR => {
            if amount != 0 {
                rotate_right_with_carry(value, amount)
            } else {
                rotate_right_extend_with_carry(value, carry_in)
            }
        }
    }
}

//P290
pub fn shift(value: u32, shift_style: ShiftStyle, amount: u32, carry_in: bool) -> u32 {
    shift_c(value, shift_style, amount, carry_in).0
}

//P43
pub fn add_with_carry(x: u32, y: u32, carry_in: bool) -> (u32, bool, bool) {
    let unsigned_sum = x + y + (carry_in as u32);
    let signed_num = (x as i32) + (y as i32) + (carry_in as i32);
    let result = unsigned_sum & !(1 << (u32::BITS - 1)); //保留后31位
    let carry_out = result != unsigned_sum;
    let overflow = (result as i32) != signed_num;
    (result, carry_out, overflow)
}

//P2368
pub fn bit_count(x: u32) -> u32 {
    let mut count = 0;
    for i in 0..32 {
        count += x >> i & 1;
    }
    count
}
