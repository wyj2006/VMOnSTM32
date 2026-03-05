use bitfield::bitfield;

use crate::{cpu::CPU, exception::Exception};

bitfield! {
    pub struct MachineStatus(u32);

    pub uie,set_uie:0;
    pub sie,set_sie:1;
    pub mie,set_mie:3;
    pub upie,set_upie:4;
    pub spie,set_spie:5;
    pub mpie,set_mpie:7;
    pub spp,set_spp:8;
    pub mpp,set_mpp:12,11;
    pub fs,set_fs:14,13;
    pub xs,set_xs:16,15;
    pub mprv,set_mprv:17;
    pub sum,set_sum:18;
    pub mxr,set_mxr:19;
    pub tvm,set_tvm:20;
    pub tw,set_tw:21;
    pub tsr,set_tsr:22;
    pub sd,set_sd:31;
}

impl Default for MachineStatus {
    fn default() -> Self {
        MachineStatus(0)
    }
}

impl CPU {
    // 偷个懒, 用for_write表示需要读还是写
    pub fn get_csr_mut(&mut self, address: u32, for_write: bool) -> Result<&mut u32, Exception> {
        let mode = address >> 8 & 0b11;
        if (self.mode as u32) < mode {
            return Err(Exception::IllegalInstruction);
        }

        match (address >> 10 & 0b11, for_write) {
            (0b01, true) | (0b10, false) => return Err(Exception::IllegalInstruction),
            _ => {}
        }

        match address {
            0x300 => Ok(&mut self.mstatus.0),
            0x304 => Ok(&mut self.mie),
            0x305 => Ok(&mut self.mtvec),
            0x340 => Ok(&mut self.mscratch),
            0x341 => Ok(&mut self.mepc),
            0x342 => Ok(&mut self.mcause),
            0x343 => Ok(&mut self.mtval),
            0x344 => Ok(&mut self.mip),
            _ => Err(Exception::IllegalInstruction),
        }
    }
}
