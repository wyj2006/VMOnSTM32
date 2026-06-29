#pragma once

#include <setjmp.h>

#include "instruction.h"

#define INT_REG_NUM 32
#define FP_REG_NUM 32
#define CSR_NUM 4096

#define MAX(x, y) ((x) > (y) ? (x) : (y))
#define MIN(x, y) ((x) < (y) ? (x) : (y))

#define MOVE(x, from, to) (*(to *)&(from){x})

#define INTERNAL_MEM_SIZE 512 * 1024
#define EXTERNAL_MEM_SIZE 1024 * 1024

typedef enum {
    InstructionAddressMisaligned,
    InstructionAccessFault,
    IllegalInstruction,
    Breakpoint,
    LoadAddressMisaligned,
    LoadAccessFault,
    StoreAddressMisaligned,
    StoreAccessFault,
    EnvironmentCallFromUMode,
    EnvironmentCallFromSMode,
    EnvironmentCallFromMMode,
    InstructionPageFault,
    LoadPageFault,
    StorePageFault,
} ExceptionCode;

typedef enum { Running, WaitingInterrupt } CPUState;

extern uint64_t int_regs[INT_REG_NUM];
extern double fp_regs[FP_REG_NUM];
extern uint64_t csrs[CSR_NUM];
extern uint64_t pc;
extern jmp_buf exception_jmp_env;
extern CPUState cpu_state;

extern uint8_t has_reserved;
extern uint64_t reserved_size;
extern uint64_t reserved_addr;
extern uint8_t is_load;

extern __attribute__((section(".ram"))) uint8_t memory[INTERNAL_MEM_SIZE];

Instruction decode(uint32_t instr);
void execute(Instruction *instruction);

uint64_t get_physical_address(uint64_t address);
uint8_t readmem8(uint64_t adderss);
void writemem8(uint64_t address, uint8_t value);
uint16_t readmem16(uint64_t address);
void writemem16(uint64_t address, uint16_t value);
uint32_t readmem32(uint64_t address);
void writemem32(uint64_t address, uint32_t value);
uint64_t readmem64(uint64_t address);
void writemem64(uint64_t address, uint64_t value);

void run();