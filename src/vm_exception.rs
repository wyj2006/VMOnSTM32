use core::convert::Infallible;
use stm32h7xx_hal::serial;

#[derive(Debug)]
pub enum VMException {
    SSofwareInterrupt,
    MSofwareInterrupt,
    STimerInterrupt,
    MTimerInterrupt,
    SExternalInterrupt,
    MExternalInterrupt,

    IllegalInstruction,
    LoadAccessFault,
    StoreAccessFault,
}

impl VMException {
    // mcause寄存器中的内容
    pub fn cause(&self) -> u32 {
        match self {
            VMException::SSofwareInterrupt => 1 << 31 | 1,
            VMException::MSofwareInterrupt => 1 << 31 | 3,
            VMException::STimerInterrupt => 1 << 31 | 5,
            VMException::MTimerInterrupt => 1 << 31 | 7,
            VMException::SExternalInterrupt => 1 << 31 | 9,
            VMException::MExternalInterrupt => 1 << 31 | 11,
            VMException::IllegalInstruction => 2,
            VMException::LoadAccessFault => 5,
            VMException::StoreAccessFault => 7,
        }
    }

    // mtval寄存器中的内容
    pub fn trap_value(&self) -> u32 {
        match self {
            _ => 0,
        }
    }
}

impl From<nb::Error<Infallible>> for VMException {
    fn from(value: nb::Error<Infallible>) -> Self {
        panic!("{value:?}")
    }
}

impl From<nb::Error<serial::Error>> for VMException {
    fn from(value: nb::Error<serial::Error>) -> Self {
        panic!("{value:?}")
    }
}
