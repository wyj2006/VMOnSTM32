use stm32h7::stm32h723::USART2;

use crate::vmerror::VMError;

pub struct ProtocolSerial;

impl ProtocolSerial {
    pub fn write(&mut self, data: u8) -> Result<(), VMError> {
        let usart = unsafe { USART2::ptr().as_ref() }.unwrap();
        while usart.isr().read().txe().bit_is_clear() {}
        // 写入数据（自动清 TXE）
        usart.tdr().write(|w| unsafe { w.tdr().bits(data as u16) });
        Ok(())
    }

    pub fn flush(&mut self) -> Result<(), VMError> {
        let usart = unsafe { USART2::ptr().as_ref() }.unwrap();
        // 等待 TC 标志
        while usart.isr().read().tc().bit_is_clear() {}
        Ok(())
    }

    pub fn read(&mut self) -> Result<u8, VMError> {
        let usart = unsafe { USART2::ptr().as_ref() }.unwrap();
        // 等待接收完成（RXNE = 1）
        while usart.isr().read().rxne().bit_is_clear() {}
        Ok(usart.rdr().read().rdr().bits() as u8)
    }
}
