use nb;
use stm32f1xx_hal::serial;
use yaxpeax_arch::ReadError;

#[derive(Debug)]
pub enum VMError {
    BusError,
    SerialError(nb::Error<serial::Error>),
}

impl From<nb::Error<serial::Error>> for VMError {
    fn from(value: nb::Error<serial::Error>) -> Self {
        VMError::SerialError(value)
    }
}

impl From<VMError> for ReadError {
    fn from(value: VMError) -> Self {
        ReadError::IOError(value.to_str())
    }
}

impl VMError {
    pub fn to_str(&self) -> &'static str {
        match self {
            VMError::BusError => "Bus Error",
            VMError::SerialError(_) => "Serial Error",
        }
    }
}
