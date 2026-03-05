use crate::{
    cpu::{CPU, Mode},
    exception::Exception,
    memory::Memory,
    register::MachineStatus,
};

pub static MEMORY_SIZE: usize = 1024;

pub struct Machine {
    pub cpu: CPU,
    pub memory: Memory,
}

impl Default for Machine {
    fn default() -> Self {
        let mut mstatus = MachineStatus::default();
        mstatus.set_mie(true);

        Machine {
            cpu: CPU {
                mie: 0xffffffff,
                mstatus,
                ..CPU::default()
            },
            memory: Memory::default(),
        }
    }
}

impl Machine {
    pub fn trap(&mut self, exception: Exception) {
        self.cpu.mcause = exception.cause();
        self.cpu.mtval = exception.trap_value();

        self.cpu.mtvec = if self.cpu.mcause >> 31 == 1 {
            //中断
            self.cpu.pc
        } else {
            //同步异常
            self.cpu.pc - 4
        };

        self.cpu.mstatus.set_mpie(self.cpu.mstatus.mie());
        self.cpu.mstatus.set_mie(false);

        self.cpu.mstatus.set_mpp(self.cpu.mode as u32);
        self.cpu.mode = Mode::Machine;
    }

    pub fn run(mut self) -> ! {
        loop {
            if let Err(e) = (|| -> Result<(), Exception> {
                let pc = self.cpu.pc as usize;
                self.cpu.pc += 4;
                let inst = self.cpu.decode(self.memory.read::<u32>(pc)?)?;
                self.execute(inst)?;
                Ok(())
            })() {
                self.trap(e);
            } else if self.cpu.mstatus.mie() && self.cpu.mie & self.cpu.mip != 0 {
                //self.trap();
            }
        }
    }
}
