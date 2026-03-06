use crate::register::MachineStatus;

pub static XREG_MAX_NUM: usize = 32;
pub static DREG_MAX_NUM: usize = 32;

#[derive(Default, Clone, Copy)]
pub enum Mode {
    User = 0b00,
    Supervisor = 0b01,
    #[default]
    Machine = 0b11,
}

impl From<u32> for Mode {
    fn from(value: u32) -> Self {
        match value {
            0b00 => Mode::User,
            0b01 => Mode::Supervisor,
            0b11 => Mode::Machine,
            _ => Mode::Machine,
        }
    }
}
#[derive(Default)]
pub struct CPU {
    pub mode: Mode,

    pub xregs: [u32; XREG_MAX_NUM],
    pub dregs: [f64; DREG_MAX_NUM],
    pub pc: u32,

    pub mtvec: u32,
    pub mepc: u32,
    pub mcause: u32,
    pub mie: u32,
    pub mip: u32,
    pub mtval: u32,
    pub mscratch: u32,
    pub mstatus: MachineStatus,
}
