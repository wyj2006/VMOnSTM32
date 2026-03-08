use core::convert::Infallible;
use stm32h7xx_hal::serial;

#[derive(Debug)]
#[repr(u32)]
pub enum VMException {
    SSofwareInterrupt,
    MSofwareInterrupt,
    STimerInterrupt,
    MTimerInterrupt,
    SExternalInterrupt,
    MExternalInterrupt,

    InstAddressMisaligned,
    InstAccessFault,
    IllegalInstruction,
    Breakpoint,
    LoadAddrMisaligned,
    LoadAccessFault,
    StoreAddrMisaligned,
    StoreAccessFault,
    ECallFromUMode,
    ECallFromSMode,
    ECallFromMMode,
    InstPageFault,
    LoadPageFault,
    StorePageFault,
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
            VMException::InstAddressMisaligned => 0,
            VMException::InstAccessFault => 1,
            VMException::IllegalInstruction => 2,
            VMException::Breakpoint => 3,
            VMException::LoadAddrMisaligned => 4,
            VMException::LoadAccessFault => 5,
            VMException::StoreAddrMisaligned => 6,
            VMException::StoreAccessFault => 7,
            VMException::ECallFromUMode => 8,
            VMException::ECallFromSMode => 9,
            VMException::ECallFromMMode => 11,
            VMException::InstPageFault => 12,
            VMException::LoadPageFault => 13,
            VMException::StorePageFault => 15,
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
