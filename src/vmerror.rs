use core::fmt;
use nb;
use yaxpeax_arch::ReadError;

#[derive(Debug)]
pub enum VMError {
    BusError,
    FmtError(fmt::Error),
    NonBlockError,
}

impl From<fmt::Error> for VMError {
    fn from(value: fmt::Error) -> Self {
        VMError::FmtError(value)
    }
}

impl From<VMError> for ReadError {
    fn from(value: VMError) -> Self {
        ReadError::IOError(value.to_str())
    }
}

impl<E> From<nb::Error<E>> for VMError {
    fn from(_value: nb::Error<E>) -> Self {
        VMError::NonBlockError
    }
}

impl VMError {
    pub fn to_str(&self) -> &'static str {
        match self {
            VMError::BusError => "Bus Error",
            VMError::FmtError(_) => "Serial Error",
            VMError::NonBlockError => "Non Blocking Error",
        }
    }
}
