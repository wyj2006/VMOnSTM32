#pragma once

#include <stdint.h>

#define CSR_SSTATUS 0x100
#define CSR_SIE 0x104
#define CSR_STVEC 0x105
#define CSR_SEPC 0x141
#define CSR_SCAUSE 0x142
#define CSR_STVAL 0x143
#define CSR_SIP 0x144
#define CSR_SATP 0x180

#define CSR_MSTATUS 0x300
#define CSR_MEDELEG 0x302
#define CSR_MIDELEG 0x303
#define CSR_MIE 0x304
#define CSR_MTVEC 0x305
#define CSR_MEPC 0x341
#define CSR_MCAUSE 0x342
#define CSR_MTVAL 0x343
#define CSR_MIP 0x344

#define CSR_FIELD_SHIFT(name, field) (name##_##field##_##SHIFT)
#define CSR_FIELD_MASK(name, field) (name##_##field##_##MASK)

#define CSR_GET_FIELD(name, field)                                             \
    ((read_csr(CSR_##name) >> CSR_FIELD_SHIFT(name, field))                    \
     & (CSR_FIELD_MASK(name, field) >> CSR_FIELD_SHIFT(name, field)))
#define CSR_SET_FIELD(name, field, value)                                      \
    ((read_csr(CSR_##name) & ~CSR_FIELD_MASK(name, field))                     \
     | (((value) << CSR_FIELD_SHIFT(name, field))                              \
        & CSR_FIELD_MASK(name, field)))

#define SSTATUS_SIE_SHIFT 1
#define SSTATUS_SIE_MASK (1 << 1)

#define SSTATUS_SPIE_SHIFT 5
#define SSTATUS_SPIE_MASK (1 << 5)

#define SSTATUS_SPP_SHIFT 8
#define SSTATUS_SPP_MASK (1 << 8)

#define SSTATUS_SUM_SHIFT 18
#define SSTATUS_SUM_MASK (1 << 18)

#define MSTATUS_MIE_SHIFT 3
#define MSTATUS_MIE_MASK (1 << 3)

#define MSTATUS_MPIE_SHIFT 7
#define MSTATUS_MPIE_MASK (1 << 7)

#define MSTATUS_MPP_SHIFT 11
#define MSTATUS_MPP_MASK (0b11 << 11)

typedef enum { User = 0b00, Supervisor = 0b01, Machine = 0b11 } PrivilegeMode;

extern PrivilegeMode privilege_mode;

uint64_t read_csr(uint16_t address);
void write_csr(uint16_t address, uint64_t value);

void trap_machine(uint64_t cause);
void trap_supervisor(uint64_t cause);