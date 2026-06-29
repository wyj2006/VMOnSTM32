#include "privilege.h"
#include "machine.h"

PrivilegeMode privilege_mode;

uint64_t read_csr(uint16_t address) { return csrs[address]; }

void write_csr(uint16_t address, uint64_t value) { csrs[address] = value; }

void trap_machine(uint64_t cause)
{
    write_csr(CSR_MEPC, pc);
    uint64_t mtvec = read_csr(CSR_MTVEC);
    switch (mtvec & 0b11)
    {
    case 0: pc = mtvec & ~0b11; break;
    case 1: pc = mtvec & ~0b11 + 4 * cause; break;
    }
    write_csr(CSR_MCAUSE, cause);

    write_csr(CSR_MTVAL, 0);

    CSR_SET_FIELD(MSTATUS, MPIE, CSR_GET_FIELD(MSTATUS, MIE));
    CSR_SET_FIELD(MSTATUS, MIE, 0);
    CSR_SET_FIELD(MSTATUS, MPP, privilege_mode);
    privilege_mode = Machine;
}

void trap_supervisor(uint64_t cause)
{
    write_csr(CSR_SEPC, pc);
    uint64_t stvec = read_csr(CSR_STVEC);
    switch (stvec & 0b11)
    {
    case 0: pc = stvec & ~0b11; break;
    case 1: pc = stvec & ~0b11 + 4 * cause; break;
    }
    write_csr(CSR_SCAUSE, cause);

    write_csr(CSR_STVAL, 0);

    CSR_SET_FIELD(SSTATUS, SPIE, CSR_GET_FIELD(SSTATUS, SIE));
    CSR_SET_FIELD(SSTATUS, SIE, 0);
    CSR_SET_FIELD(SSTATUS, SPP, privilege_mode);
    privilege_mode = Supervisor;
}