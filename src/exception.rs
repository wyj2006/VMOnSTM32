use core::convert::Infallible;
use stm32h7xx_hal::serial;

#[derive(Debug)]
pub enum Exception {
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

impl Exception {
    // mcause寄存器中的内容
    pub fn cause(&self) -> u32 {
        match self {
            Exception::SSofwareInterrupt => 1 << 31 | 1,
            Exception::MSofwareInterrupt => 1 << 31 | 3,
            Exception::STimerInterrupt => 1 << 31 | 5,
            Exception::MTimerInterrupt => 1 << 31 | 7,
            Exception::SExternalInterrupt => 1 << 31 | 9,
            Exception::MExternalInterrupt => 1 << 31 | 11,
            Exception::IllegalInstruction => 2,
            Exception::LoadAccessFault => 5,
            Exception::StoreAccessFault => 7,
        }
    }

    // mtval寄存器中的内容
    pub fn trap_value(&self) -> u32 {
        match self {
            _ => 0,
        }
    }
}

impl From<nb::Error<Infallible>> for Exception {
    fn from(value: nb::Error<Infallible>) -> Self {
        panic!("{value:?}")
    }
}

impl From<nb::Error<serial::Error>> for Exception {
    fn from(value: nb::Error<serial::Error>) -> Self {
        panic!("{value:?}")
    }
}
