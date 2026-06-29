#include "machine.h"
#include "privilege.h"

CPUState cpu_state = Running;
jmp_buf exception_jmp_env;

void run()
{
    while (1)
    {
        if (cpu_state == Running)
        {
            Instruction instruction = decode(readmem32(pc));
            pc += 4;
            execute(&instruction);
        }

        uint8_t exception = setjmp(exception_jmp_env);
        if (exception == 0)
        {
            uint64_t mie = read_csr(CSR_MIE);
            uint64_t mip = read_csr(CSR_MIP);
            uint64_t mideleg = read_csr(CSR_MIDELEG);
            uint64_t sie = read_csr(CSR_SIE);
            uint64_t sip = read_csr(CSR_SIP);
            for (int i = 1; i <= 11; i += 2)
            {
                if ((((mie >> i) & (mip >> i)) & 1) == 1)
                {
                    if (cpu_state == WaitingInterrupt) cpu_state = Running;
                    if (CSR_GET_FIELD(MSTATUS, MIE) == 1)
                    {
                        uint64_t cause = (1ull << 63) | i;
                        // 只能陷入到更高或同级的模式
                        switch (privilege_mode)
                        {
                        case Machine: trap_machine(cause); break;
                        case Supervisor:
                        case User:
                            if (((mideleg >> i) & 1) == 1)
                            {
                                if (CSR_GET_FIELD(SSTATUS, SIE) == 1
                                    && (((sie >> i) & (sip >> i)) & 1) == 1)
                                    trap_supervisor(cause);
                            }
                            else
                                trap_machine(cause);
                            break;
                        }
                        break;
                    }
                }
            }
        }
        else
        {
            // 跟实际的exception code差了1
            uint64_t code = exception - 1;
            uint64_t medeleg = read_csr(CSR_MEDELEG);
            if (((medeleg >> code) & 1) == 1)
                trap_supervisor(code);
            else
                trap_machine(code);
        }
    }
}