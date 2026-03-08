extern crate alloc;

use crate::vm_exception::VMException;
use alloc::collections::BTreeMap;
use bitfield::bitfield;

pub static XREG_MAX_NUM: usize = 32;
pub static DREG_MAX_NUM: usize = 32;

#[derive(Default, Clone, Copy, PartialEq, PartialOrd)]
#[repr(u32)]
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

bitfield! {
    pub struct MStatus(u32);

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

bitfield! {
    pub struct SStatus(u32);

    pub uie,set_uie:0;
    pub sie,set_sie:1;
    pub upie,set_upie:4;
    pub spie,set_spie:5;
    pub spp,set_spp:8;
    pub fs,set_fs:14,13;
    pub xs,set_xs:16,15;
    pub sum,set_sum:18;
    pub mxr,set_mxr:19;
    pub sd,set_sd:31;
}

bitfield! {
    pub struct SATP(u32);

    pub ppn,set_ppn:21,0;
    pub asid,set_asid:30,22;
    pub mode,set_mode:31;
}

pub struct CPU {
    pub mode: Mode,

    pub xregs: [u32; XREG_MAX_NUM],
    pub dregs: [f64; DREG_MAX_NUM],
    pub pc: u32,

    pub csrs: BTreeMap<u32, u32>,
}

impl CPU {
    pub fn get_csr(&self, address: u32) -> Result<u32, VMException> {
        let mode = address >> 8 & 0b11;
        if (self.mode as u32) < mode {
            return Err(VMException::IllegalInstruction);
        }

        match address >> 10 & 0b11 {
            0b10 => return Err(VMException::IllegalInstruction),
            _ => {}
        }

        if address == CSRAddress::sie_raw as u32 {
            Ok(self.sie())
        } else if address == CSRAddress::sip_raw as u32 {
            Ok(self.sip())
        } else if let Some(t) = self.csrs.get(&address) {
            Ok(*t)
        } else {
            Err(VMException::IllegalInstruction)
        }
    }

    pub fn set_csr(&mut self, address: u32, value: u32) -> Result<(), VMException> {
        let mode = address >> 8 & 0b11;
        if (self.mode as u32) < mode {
            return Err(VMException::IllegalInstruction);
        }

        match address >> 10 & 0b11 {
            0b01 => return Err(VMException::IllegalInstruction),
            _ => {}
        }

        if address == CSRAddress::sie_raw as u32 {
            self.set_mie(value);
        } else if address == CSRAddress::sip_raw as u32 {
            self.set_sip(value);
        } else if let Some(t) = self.csrs.get_mut(&address) {
            *t = value;
        } else {
            return Err(VMException::IllegalInstruction);
        }

        Ok(())
    }

    pub fn sie(&self) -> u32 {
        self.mie() & self.mideleg()
    }

    pub fn set_sie(&mut self, value: u32) {
        self.set_mie(self.mie() & !self.mideleg() | value & self.mideleg());
    }

    pub fn sip(&self) -> u32 {
        self.mip() & self.mideleg()
    }

    pub fn set_sip(&mut self, value: u32) {
        self.set_mip(self.mip() & !self.mideleg() | value & self.mideleg());
    }
}

macro_rules! csr_getter_setter {
    ($($name:ident,$set_name:ident = $val:expr ;)+) => {
        #[allow(non_camel_case_types)]
        #[repr(u32)]
        pub enum CSRAddress{
            $($name = $val,)+
        }


        impl Default for CPU{
            fn default()->CPU{
                let mut csrs=BTreeMap::new();

                $(csrs.insert($val,0);)+

                CPU{
                    mode:Mode::default(),
                    xregs: [0; XREG_MAX_NUM],
                    dregs: [0.; DREG_MAX_NUM],
                    pc: 0,
                    csrs
                }
            }
        }

        impl CPU{
            $(
                pub fn $name(&self)->u32{
                    *self.csrs.get(&$val).unwrap()
                }

                pub fn $set_name(&mut self,value:u32){
                    *self.csrs.get_mut(&$val).unwrap()=value;
                }
            )+
        }
    };
}

csr_getter_setter!(
    fflags,set_fflags=0x001;
    frm,set_frm=0x002;
    fcsr,set_fcsr=0x003;

    cycle,set_cycle=0xc00;
    time,set_time=0xc01;
    instret,set_instret=0xc02;
    hpmcounter3,set_hpmcounter3=0xc03;
    //TODO hpmcounter4..31
    cycleh,set_cycleh=0xc80;
    timeh,set_timeh=0xc81;
    instreth,set_instreth=0xc82;
    hpmcounter3h,set_hpmcounter3h=0xc83;
    //TODO hpmcounter4..31h

    sstatus,set_sstatus=0x100;
    sie_raw,set_sie_raw=0x104;
    stvec,set_stvec=0x105;
    scounteren,set_scounteren=0x106;

    senvcfg,set_senvcfg=0x10a;

    sscratch,set_sscratch=0x140;
    sepc,set_sepc=0x141;
    scause,set_scause=0x142;
    stval,set_stval=0x143;
    sip_raw,set_sip_raw=0x144;

    satp,set_satp=0x180;

    scontext,set_scontext=0x5a8;

    hstatus,set_hstatus=0x600;
    hedeleg,set_hedeleg=0x602;
    hideleg,set_hideleg=0x603;
    hie,set_hie=0x604;
    hcounteren,set_hcounteren=0x606;
    hgeie,set_hgeie=0x607;

    htval,set_htval=0x643;
    hip,set_hip=0x644;
    hvip,set_hvip=0x645;
    htinst,set_htinst=0x64a;
    hgeip,set_hgeip=0xe12;

    henvcfg,set_henvcfg=0x60a;
    henvcfgh,set_henvcfgh=0x61a;

    hgatp,set_hgatp=0x680;

    hcontext,set_hcontext=0x6a8;

    htimedelta,set_htimedelta=0x605;
    htimedeltah,set_htimedeltah=0x615;

    vsstatus,set_vsstatus=0x200;
    vsie,set_vsie=0x204;
    vstvec,set_vstvec=0x205;
    vsscratch,set_vsscratch=0x240;
    vsepc,set_vsepc=0x241;
    vscause,set_vscause=0x242;
    vstval,set_vstval=0x243;
    vsip,set_vsip=0x244;
    vsatp,set_vsatp=0x280;

    mvendorid,set_mvendorid=0xf11;
    marchid,set_marchid=0xf12;
    mimpid,set_mimpid=0xf13;
    mhartid,set_mhartid=0xf14;
    mconfigptr,set_mconfigptr=0xf15;

    mstatus,set_mstatus = 0x300;
    misa,set_misa = 0x301;
    medeleg,set_medeleg = 0x302;
    mideleg,set_mideleg = 0x303;
    mie,set_mie = 0x304;
    mtvec,set_mtvec = 0x305;
    mcounteren,set_mcounteren = 0x306;
    mstatush,set_mstatush = 0x310;

    mscratch,set_mscratch = 0x340;
    mepc,set_mepc = 0x341;
    mcause,set_mcause = 0x342;
    mtval,set_mtval = 0x343;
    mip,set_mip = 0x344;
    mtinst,set_mtinst = 0x34a;
    mtval2,set_mtval2 = 0x34b;

    menvcfg,set_menvcfg=0x30a;
    menvcfgh,set_menvcfgh=0x31a;
    mseccfg,set_mseccfg=0x747;
    mseccfgh,set_mseccfgh=0x757;

    pmpcfg0,set_pmpcfg0=0x3a0;
    //TODO pmpcfg1..63
);
