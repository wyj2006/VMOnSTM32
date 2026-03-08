use crate::{
    cpu::{CPU, MStatus, Mode, SStatus},
    memory::Memory,
    vm_exception::VMException,
};

pub static MEMORY_SIZE: usize = 1024;

pub struct Machine {
    pub cpu: CPU,
    pub memory: Memory,
}

impl Default for Machine {
    fn default() -> Self {
        let mut mstatus = MStatus(0);
        mstatus.set_mie(true);

        let mut cpu = CPU::default();
        cpu.set_mie(0xffffffff);
        cpu.set_mstatus(mstatus.0);

        Machine {
            cpu,
            memory: Memory::default(),
        }
    }
}

impl Machine {
    pub fn trap_m(&mut self, exception: VMException) {
        self.cpu.set_mcause(exception.cause());
        self.cpu.set_mtvec(exception.trap_value());

        self.cpu.set_mepc(if self.cpu.mcause() >> 31 == 1 {
            //中断
            self.cpu.pc
        } else {
            //同步异常
            self.cpu.pc - 4
        });

        let base = self.cpu.mtvec() >> 2;
        let mode = self.cpu.mtvec() & 0b11;
        self.cpu.pc = if mode == 0 {
            base
        } else if mode == 1 {
            if self.cpu.mcause() >> 31 == 1 {
                base + 4 * self.cpu.mcause() & 0x7fffffff
            } else {
                base
            }
        } else {
            unreachable!()
        };

        let mut mstatus = MStatus(self.cpu.mstatus());

        mstatus.set_mpie(mstatus.mie());
        mstatus.set_mie(false);
        mstatus.set_mpp(self.cpu.mode as u32);
        self.cpu.set_mstatus(mstatus.0);

        self.cpu.mode = Mode::Machine;
    }

    pub fn trap_s(&mut self, exception: VMException) {
        self.cpu.set_scause(exception.cause());
        self.cpu.set_stvec(exception.trap_value());

        self.cpu.set_sepc(if self.cpu.scause() >> 31 == 1 {
            //中断
            self.cpu.pc
        } else {
            //同步异常
            self.cpu.pc - 4
        });

        let base = self.cpu.stvec() >> 2;
        let mode = self.cpu.stvec() & 0b11;
        self.cpu.pc = if mode == 0 {
            base
        } else if mode == 1 {
            if self.cpu.scause() >> 31 == 1 {
                base + 4 * self.cpu.scause() & 0x7fffffff
            } else {
                base
            }
        } else {
            unreachable!()
        };

        let mut sstatus = SStatus(self.cpu.sstatus());

        sstatus.set_spie(sstatus.sie());
        sstatus.set_sie(false);
        sstatus.set_spp(self.cpu.mode as u32 != 0);
        self.cpu.set_sstatus(sstatus.0);

        self.cpu.mode = Mode::Supervisor;
    }

    pub fn run(mut self) -> ! {
        loop {
            if let Err(e) = (|| -> Result<(), VMException> {
                let pc = self.cpu.pc as usize;
                self.cpu.pc += 4;
                let inst = self.cpu.decode(self.memory.read::<u32>(pc)?)?;
                self.execute(inst)?;
                Ok(())
            })() {
                self.trap_m(e);
            } else if MStatus(self.cpu.mstatus()).mie() && self.cpu.mie() & self.cpu.mip() != 0 {
                let mie = self.cpu.mie();
                let mip = self.cpu.mip();
                let mideleg = self.cpu.mideleg();

                for (i, exception) in [
                    (1, VMException::SSofwareInterrupt),
                    (3, VMException::MSofwareInterrupt),
                    (5, VMException::STimerInterrupt),
                    (7, VMException::MTimerInterrupt),
                    (9, VMException::SExternalInterrupt),
                    (11, VMException::MExternalInterrupt),
                ] {
                    if mip >> i & 1 == 1 && mie >> i & 1 == 1 {
                        //不能陷入到级别更低的模式中
                        if self.cpu.mode < Mode::Machine && mideleg >> i & 1 == 1 {
                            if SStatus(self.cpu.sstatus()).sie() {
                                self.trap_s(exception);
                                break;
                            }
                        } else {
                            self.trap_m(exception);
                            break;
                        }
                    }
                }
            }
        }
    }
}
