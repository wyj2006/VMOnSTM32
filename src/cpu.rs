use bitfield::bitfield;

use crate::machine::Machine;

//P45
pub const SP_INDEX: usize = 13;
pub const LR_INDEX: usize = 14;
pub const PC_INDEX: usize = 15;

/* P49
Application Program Status Register
31 30 29 28 27 26      24 23               20 19     16 15                   0
 N  Z  C  V  Q| RAZ/SBZP | Reserved,UNK/SBZP | GE[3:0] | Reserved, UNKNOWN/SBZP
*/
bitfield! {
    pub struct APSRegister(u32);
    pub n, _: 31;
    pub z, _: 30;
    pub c, _: 29;
    pub v, _: 28;
    pub q, _: 27;
    pub ge, _: 19, 16;
}

pub struct APSRegisterMut<'a>(&'a mut CPSRegister);

/* P51
IT block state register, ITSTATE
*/
bitfield! {
    pub struct ITRegister(u8);
}

pub struct ITRegisterMut<'a>(&'a mut CPSRegister);

/* P50
Instruction set state register, ISETSTATE
1 0
J T

J T Instruction set state
0 0 ARM
0 1 Thumb
1 0 Jazelle
1 1 ThumbEE
*/
bitfield! {
    pub struct ISetRegister(u8);
    pub j, _: 1;
    pub t, _: 0;
}

pub struct ISetRegisterMut<'a>(&'a mut CPSRegister);

pub enum InstrSet {
    Arm,
    Thumb,
    Jazelle,
    ThumbEE,
}

/* P1148
Current Program Status Register
31 30 29 28 27  26     25 24 23               20 19     16 15     10  9 8 7 6 5  4 3 2 1 0
 N  Z  C  V  Q | IT[1:0] | J| Reserved,RAZ/SBZP | GE[3:0] | IT[7:2] | E A I F T |  M[4:0]
Condition flags|                                                    | Mask Bits |
*/
bitfield! {
    pub struct CPSRegister(u32);
    pub n, set_n: 31;
    pub z, set_z: 30;
    pub c, set_c: 29;
    pub v, set_v: 28;
    pub q, set_q: 27;
    pub it_low, set_it_low: 26,25;
    pub j, set_j: 24;
    pub ge, set_ge: 19, 16;
    pub it_high, set_it_high: 15,10;
    pub e, set_e: 9;
    pub a, set_a: 8;
    pub i, set_i: 7;
    pub f, set_f: 6;
    pub t, set_t: 5;
    pub m, set_m: 4,0;
}

pub struct CPU {
    pub regs: [u32; 16],
    pub cpsr: CPSRegister,
}

impl Default for CPU {
    fn default() -> Self {
        CPU {
            regs: [0; 16],
            cpsr: CPSRegister::default(),
        }
    }
}

impl CPU {
    pub fn apsr(&self) -> APSRegister {
        APSRegister(self.cpsr.0 & 0b11111_00_0_0000_1111_000000_00000_00000)
    }

    pub fn apsr_mut(&mut self) -> APSRegisterMut<'_> {
        APSRegisterMut(&mut self.cpsr)
    }

    pub fn it_state(&self) -> ITRegister {
        ITRegister(self.cpsr.it())
    }

    pub fn it_state_mut(&mut self) -> ITRegisterMut<'_> {
        ITRegisterMut(&mut self.cpsr)
    }

    pub fn iset_state(&self) -> ISetRegister {
        ISetRegister((self.cpsr.j() as u8) << 1 | self.cpsr.t() as u8)
    }

    pub fn iset_state_mut(&mut self) -> ISetRegisterMut<'_> {
        ISetRegisterMut(&mut self.cpsr)
    }
}

impl CPSRegister {
    pub fn it(&self) -> u8 {
        (self.it_high() << 2 | self.it_low()) as u8
    }

    pub fn set_it(&mut self, bit: u8) {
        self.set_it_low((bit & 0b11) as u32);
        self.set_it_high((bit >> 2) as u32);
    }
}

impl Default for CPSRegister {
    fn default() -> Self {
        //P1205
        let mut cpsr = CPSRegister(0);
        cpsr.set_m(0b10011);
        cpsr.set_i(true);
        cpsr.set_f(true);
        cpsr.set_a(true);
        cpsr.set_j(false);
        cpsr.set_t(false); // ARM
        cpsr.set_e(false); // little-endian
        cpsr
    }
}

impl ISetRegisterMut<'_> {
    pub fn set_value(&mut self, value: u8) {
        self.0.set_j(value >> 1 & 1 == 1);
        self.0.set_t(value & 1 == 1);
    }
}

impl APSRegisterMut<'_> {
    pub fn set_n(&mut self, value: bool) {
        self.0.set_n(value);
    }

    pub fn set_z(&mut self, value: bool) {
        self.0.set_z(value);
    }

    pub fn set_c(&mut self, value: bool) {
        self.0.set_c(value);
    }

    pub fn set_v(&mut self, value: bool) {
        self.0.set_v(value);
    }

    pub fn set_q(&mut self, value: bool) {
        self.0.set_q(value);
    }

    pub fn set_ge(&mut self, value: u32) {
        self.0.set_ge(value);
    }
}

impl ITRegisterMut<'_> {
    pub fn set_value(&mut self, value: u8) {
        self.0.set_it(value);
    }
}

impl Machine {
    //P52
    pub fn in_it_block(&self) -> bool {
        self.cpu.it_state().0 & 0b1111 != 0b0000
    }

    //P51
    pub fn current_instr_set(&self) -> InstrSet {
        match self.cpu.iset_state().0 {
            0b00 => InstrSet::Arm,
            0b01 => InstrSet::Thumb,
            0b10 => InstrSet::Jazelle,
            0b11 => InstrSet::ThumbEE,
            _ => unreachable!(),
        }
    }

    //P51
    pub fn select_instr_set(&mut self, iset: InstrSet) {
        let mut iset_state = self.cpu.iset_state_mut();
        match iset {
            InstrSet::Arm => iset_state.set_value(0b00),
            InstrSet::Thumb => iset_state.set_value(0b01),
            InstrSet::Jazelle => iset_state.set_value(0b10),
            InstrSet::ThumbEE => iset_state.set_value(0b11),
        }
    }
}
