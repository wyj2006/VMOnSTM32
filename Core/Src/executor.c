#include <math.h>
#include <string.h>

#include "machine.h"
#include "privilege.h"

#define DOUBLE_SIGN_MASK ((1ull << 63) - 1)
#define FLOAT_SIGN_MASK ((1ull << 31) - 1)

uint64_t int_regs[INT_REG_NUM];
double fp_regs[FP_REG_NUM];
uint64_t csrs[CSR_NUM];
uint64_t pc;

uint64_t read_operand(Operand *operand)
{
    switch (operand->kind)
    {
    case Nothing: return 0;
    case Immediate: return operand->imm;
    case IntReg: return int_regs[operand->reg_index];
    case FPReg: return *(uint64_t *)&(fp_regs[operand->reg_index]);
    case CSR: return read_csr(operand->csr_index);
    }
}

void write_operand(Operand *operand, uint64_t value)
{
    switch (operand->kind)
    {
    case Nothing: break;
    case Immediate: break;
    case IntReg: int_regs[operand->reg_index] = value; break;
    case FPReg:
        memcpy(&fp_regs[operand->reg_index], &value, sizeof(value));
        break;
    case CSR: write_csr(operand->csr_index, value); break;
    }
}

void execute(Instruction *instruction)
{
    uint64_t t;
    Operand *rd = &instruction->operands[0];

    uint64_t a = read_operand(&instruction->operands[0]);
    uint64_t b = read_operand(&instruction->operands[1]);
    uint64_t c = read_operand(&instruction->operands[2]);
    uint64_t d = read_operand(&instruction->operands[3]);

    // float fa = *(float *)&a;
    float fb = *(float *)&b;
    float fc = *(float *)&c;
    float fd = *(float *)&d;

    // double da = *(double *)&a;
    double db = *(double *)&b;
    double dc = *(double *)&c;
    double dd = *(double *)&d;

    switch (instruction->opcode)
    {
    case Add: write_operand(rd, b + c); break;
    case Add_W: write_operand(rd, (int64_t)((int32_t)b + (int32_t)c)); break;
    case AMO_Add_D:
        t = readmem64(b);
        writemem64(b, t + c);
        write_operand(rd, t);
        break;
    case AMO_Add_W:
        t = readmem32(b);
        writemem32(b, t + c);
        write_operand(rd, t);
        break;
    case AMO_And_D:
        t = readmem64(b);
        writemem64(b, t & c);
        write_operand(rd, t);
        break;
    case AMO_And_W:
        t = readmem32(b);
        writemem32(b, t & c);
        write_operand(rd, t);
        break;
    case AMO_Max_D:
        t = readmem64(b);
        writemem64(b, MAX((int64_t)t, (int64_t)c));
        write_operand(rd, t);
        break;
    case AMO_Max_W:
        t = readmem32(b);
        writemem32(b, MAX((int64_t)t, (int64_t)c));
        write_operand(rd, t);
        break;
    case AMO_Max_UD:
        t = readmem64(b);
        writemem64(b, MAX(t, c));
        write_operand(rd, t);
        break;
    case AMO_Max_UW:
        t = readmem32(b);
        writemem32(b, MAX(t, c));
        write_operand(rd, t);
        break;
    case AMO_Min_D:
        t = readmem64(b);
        writemem64(b, MIN((int64_t)t, (int64_t)c));
        write_operand(rd, t);
        break;
    case AMO_Min_W:
        t = readmem32(b);
        writemem32(b, MIN((int64_t)t, (int64_t)c));
        write_operand(rd, t);
        break;
    case AMO_Min_UD:
        t = readmem64(b);
        writemem64(b, MIN(t, c));
        write_operand(rd, t);
        break;
    case AMO_Min_UW:
        t = readmem32(b);
        writemem32(b, MIN(t, c));
        write_operand(rd, t);
        break;
    case AMO_Or_D:
        t = readmem64(b);
        writemem64(b, t | c);
        write_operand(rd, t);
        break;
    case AMO_Or_W:
        t = readmem32(b);
        writemem32(b, t | c);
        write_operand(rd, t);
        break;
    case AMO_Swap_D:
        t = readmem64(b);
        writemem64(b, c);
        write_operand(rd, t);
        break;
    case AMO_Swap_W:
        t = readmem64(b);
        writemem64(b, c);
        write_operand(rd, t);
        break;
    case AMO_Xor_D:
        t = readmem64(b);
        writemem64(b, t ^ c);
        write_operand(rd, t);
        break;
    case AMO_Xor_W:
        t = readmem32(b);
        writemem32(b, t ^ c);
        write_operand(rd, t);
        break;
    case And: write_operand(rd, b & c); break;
    case AUIPC: write_operand(rd, pc + b); break;
    case BEq:
        if (a == b) pc += c;
        break;
    case BGe:
        if ((int64_t)a >= (int64_t)b) pc += c;
        break;
    case BGe_U:
        if (a >= b) pc += c;
        break;
    case BLt:
        if ((int64_t)a < (int64_t)b) pc += c;
        break;
    case BLt_U:
        if (a < b) pc += c;
        break;
    case BNe:
        if (a != b) pc += c;
        break;
    case CSR_RC:
        write_operand(&instruction->operands[1], b & ~c);
        write_operand(rd, b);
        break;
    case CSR_RS:
        write_operand(&instruction->operands[1], b | c);
        write_operand(rd, b);
        break;
    case CSR_RW:
        write_operand(&instruction->operands[1], c);
        write_operand(rd, b);
        break;
    case Div: write_operand(rd, (int64_t)b / (int64_t)c); break;
    case Div_U: write_operand(rd, b / c); break;
    case Div_UW:
        write_operand(rd, (int64_t)(int32_t)((uint32_t)b / (uint32_t)c));
        break;
    case Div_W: write_operand(rd, (int64_t)((int32_t)b / (int32_t)c)); break;
    case EBreak: longjmp(exception_jmp_env, Breakpoint + 1); break;
    case ECall:
        switch (privilege_mode)
        {
        case User:
            longjmp(exception_jmp_env, EnvironmentCallFromUMode + 1);
            break;
        case Supervisor:
            longjmp(exception_jmp_env, EnvironmentCallFromSMode + 1);
            break;
        case Machine:
            longjmp(exception_jmp_env, EnvironmentCallFromMMode + 1);
            break;
        }
        break;
    case FAdd_D: write_operand(rd, MOVE(db + dc, double, uint64_t)); break;
    case FAdd_S: write_operand(rd, MOVE(fb + fc, float, uint64_t)); break;
    case FClass_D: {
        t = b;
        uint32_t sign = (t >> 63) & 1ull;
        uint32_t exp = (t >> 52) & 0x7ffull;
        uint64_t frac = t & 0x000fffffffffffffull;
        if (exp == 0x7FF)
        {
            if (frac == 0) // -oo 或 +oo
                write_operand(rd, sign ? (1 << 0) : (1 << 7));
            else
            {
                if ((frac & 0x0008000000000000ull) == 0) // signaling NaN
                    write_operand(rd, 1 << 8);
                else // quiet NaN
                    write_operand(rd, 1 << 9);
            }
        }
        else if (exp == 0)
        {
            if (frac == 0) // -0.0 或 +0.0
                write_operand(rd, sign ? (1 << 3) : (1 << 4));
            else // 负/正非规格化数
                write_operand(rd, sign ? (1 << 2) : (1 << 5));
        }
        else // 规格化数
            write_operand(rd, sign ? (1 << 1) : (1 << 6));
    }
    break;
    case FClass_S: {
        uint32_t t = b;
        uint32_t sign = (t >> 31) & 1;
        uint32_t exp = (t) & 0xFF;
        uint32_t frac = t & 0x7FFFFF;
        if (exp == 0xFF)
        {
            if (frac == 0) // -oo 或 +oo
                write_operand(rd, sign ? (1 << 0) : (1 << 7));
            else
            {
                if ((frac & 0x400000) == 0) // signaling NaN
                    write_operand(rd, 1 << 8);
                else // quiet NaN
                    write_operand(rd, 1 << 9);
            }
        }
        else if (exp == 0)
        {
            if (frac == 0) // -0.0 或 +0.0
                write_operand(rd, sign ? (1 << 3) : (1 << 4));
            else // 非规格化数
                write_operand(rd, sign ? (1 << 2) : (1 << 5));
        }
        else // 规格化数
            write_operand(rd, sign ? (1U << 1) : (1U << 6));
    }
    break;
    case FCvt_D_L: write_operand(rd, MOVE((int64_t)b, double, uint64_t)); break;
    case FCvt_D_LU: write_operand(rd, MOVE(b, double, uint64_t)); break;
    case FCvt_D_S: write_operand(rd, MOVE(fb, double, uint64_t)); break;
    case FCvt_D_W: write_operand(rd, MOVE((int32_t)b, double, uint64_t)); break;
    case FCvt_D_WU:
        write_operand(rd, MOVE((uint32_t)b, double, uint64_t));
        break;
    case FCvt_L_D: write_operand(rd, (int64_t)db); break;
    case FCvt_L_S: write_operand(rd, (int64_t)fb);
    case FCvt_LU_D: write_operand(rd, db); break;
    case FCvt_LU_S: write_operand(rd, fb); break;
    case FCvt_S_D: write_operand(rd, MOVE(db, float, uint64_t)); break;
    case FCvt_S_L: write_operand(rd, MOVE((int64_t)b, float, uint64_t)); break;
    case FCvt_S_LU: write_operand(rd, MOVE(b, float, uint64_t)); break;
    case FCvt_S_W: write_operand(rd, MOVE((int32_t)b, float, uint64_t)); break;
    case FCvt_S_WU:
        write_operand(rd, MOVE((uint32_t)b, float, uint64_t));
        break;
    case FCvt_W_D: write_operand(rd, (int64_t)(int32_t)db); break;
    case FCvt_WU_D: write_operand(rd, (int64_t)(int32_t)(uint32_t)db); break;
    case FCvt_W_S: write_operand(rd, (int64_t)(int32_t)fb); break;
    case FCvt_WU_S: write_operand(rd, (int64_t)(int32_t)(uint32_t)fb); break;
    case FDiv_D: write_operand(rd, MOVE(db / dc, double, uint64_t));
    case FDiv_S: write_operand(rd, MOVE(fb / fc, float, uint64_t));
    // 模拟器没有流水线, 无需同步
    case Fence: break;
    case FenceI: break;
    case FEq_D: write_operand(rd, db == dc); break;
    case FEq_S: write_operand(rd, fb == fc); break;
    case FLoad_D: write_operand(rd, readmem64(b + c)); break;
    case FLe_D: write_operand(rd, db <= dc); break;
    case FLe_S: write_operand(rd, fb <= fc); break;
    case FLt_D: write_operand(rd, db < dc); break;
    case FLt_S: write_operand(rd, fb < fc); break;
    case FLoad_W: write_operand(rd, readmem32(b + c)); break;
    case FMulAdd_D:
        write_operand(rd, MOVE(db * dc + dd, double, uint64_t));
        break;
    case FMulAdd_S:
        write_operand(rd, MOVE(fb * fc + fd, float, uint64_t));
        break;
    case FMax_D: write_operand(rd, MAX(db, dc)); break;
    case FMax_S: write_operand(rd, MAX(fb, fc)); break;
    case FMin_D: write_operand(rd, MIN(db, dc)); break;
    case FMin_S: write_operand(rd, MIN(fb, fc)); break;
    case FMulSub_D:
        write_operand(rd, MOVE(db * dc - dd, double, uint64_t));
        break;
    case FMulSub_S:
        write_operand(rd, MOVE(fb * fc - fd, float, uint64_t));
        break;
    case FMul_D: write_operand(rd, MOVE(db * dc, double, uint64_t)); break;
    case FMul_S: write_operand(rd, MOVE(fb * fc, float, uint64_t)); break;
    case FMove_D_X: write_operand(rd, b); break;
    case FMove_S_X: write_operand(rd, (uint32_t)b); break;
    case FMove_X_D: write_operand(rd, b); break;
    case FMove_X_W: write_operand(rd, (uint32_t)b); break;
    case FNegMulAdd_D:
        write_operand(rd, MOVE(-(db * dc) + dd, double, uint64_t));
        break;
    case FNegMulAdd_S:
        write_operand(rd, MOVE(-(fb * fc) + fd, float, uint64_t));
        break;
    case FNegMulSub_D:
        write_operand(rd, MOVE(-(db * dc) - dd, double, uint64_t));
        break;
    case FNegMulSub_S:
        write_operand(rd, MOVE(-(fb * fc) - fd, float, uint64_t));
        break;
    case FStore_D: writemem64(b + c, a); break;
    case FSgnJ_D:
        write_operand(rd, MOVE((c & ~DOUBLE_SIGN_MASK) | b & DOUBLE_SIGN_MASK,
                               double, uint64_t));
        break;
    case FSgnJ_S:
        write_operand(rd, MOVE((c & ~FLOAT_SIGN_MASK) | b & FLOAT_SIGN_MASK,
                               float, uint64_t));
        break;
    case FSgnJ_Neg_D:
        write_operand(rd, MOVE(~(c & ~DOUBLE_SIGN_MASK) | b & DOUBLE_SIGN_MASK,
                               double, uint64_t));
        break;
    case FSgnJ_Neg_S:
        write_operand(rd, MOVE(~(c & ~FLOAT_SIGN_MASK) | b & FLOAT_SIGN_MASK,
                               float, uint64_t));
        break;
    case FSgnJ_Xor_D:
        write_operand(rd, MOVE((c & ~DOUBLE_SIGN_MASK) ^ (b & ~DOUBLE_SIGN_MASK)
                                   | b & DOUBLE_SIGN_MASK,
                               double, uint64_t));
        break;
    case FSgnJ_Xor_S:
        write_operand(rd, MOVE((c & ~FLOAT_SIGN_MASK) ^ (b & ~FLOAT_SIGN_MASK)
                                   | b & FLOAT_SIGN_MASK,
                               float, uint64_t));
        break;
    case FSqrt_D: write_operand(rd, MOVE(sqrt(db), double, uint64_t)); break;
    case FSqrt_S: write_operand(rd, MOVE(sqrtf(fb), float, uint64_t)); break;
    case FSub_D: write_operand(rd, MOVE(db - dc, double, uint64_t)); break;
    case FSub_S: write_operand(rd, MOVE(fb - fc, float, uint64_t)); break;
    case FStore_W: writemem32(b + c, a); break;
    case JAL:
        write_operand(rd, pc + 4);
        pc += b;
        break;
    case JAL_Reg:
        write_operand(rd, pc + 4);
        pc = (b + c) & ~1;
        break;
    case Load_B: write_operand(rd, (int64_t)(int8_t)readmem8(b + c));
    case Load_BU: write_operand(rd, readmem8(b + c)); break;
    case Load_Reserved_D:
        is_load = 1;
        has_reserved = 1;
        reserved_size = 8;
        reserved_addr = get_physical_address(b + c);
    case Load_D: write_operand(rd, readmem64(b + c)); break;
    case Load_H: write_operand(rd, (int64_t)(int16_t)readmem16(b + c)); break;
    case Load_HU: write_operand(rd, readmem16(b + c)); break;
    case Load_Reserved_W:
        is_load = 1;
        has_reserved = 1;
        reserved_size = 4;
        reserved_addr = get_physical_address(b + c);
    case Load_W: write_operand(rd, (int64_t)(int32_t)readmem32(b + c)); break;
    case Load_WU: write_operand(rd, readmem32(b + c)); break;
    case Load_Imm: write_operand(rd, b); break;
    case MRet:
        pc = read_csr(CSR_MEPC);
        CSR_SET_FIELD(MSTATUS, MIE, CSR_GET_FIELD(MSTATUS, MPIE));
        privilege_mode = CSR_GET_FIELD(MSTATUS, MPP);
        break;
    case Mul: write_operand(rd, b * c); break;
    case Mul_H: {
        uint64_t b_lo = b & 0xffffffff;
        uint64_t b_hi = b >> 32;
        uint64_t c_lo = c & 0xffffffff;
        uint64_t c_hi = c_lo >> 32;
        /*
                   b_hi        b_lo
                x  c_hi        c_lo
                ----------------------
                   b_hi*c_lo   c_lo*b_lo
        c_hi*b_hi  c_hi*b_lo
        */
        uint64_t lo_lo = c_lo * b_lo;
        uint64_t lo_hi = c_lo * b_hi;
        uint64_t hi_lo = c_hi * b_lo;
        uint64_t hi_hi = c_hi * b_hi;
        uint64_t mid = lo_hi + hi_lo + (lo_lo >> 32);
        uint64_t high = hi_hi + (mid >> 32);

        if ((int64_t)b < 0) high -= c;
        if ((int64_t)c < 0) high -= b;

        write_operand(rd, high);
        break;
    }
    case Mul_H_SU: {
        uint64_t b_lo = b & 0xffffffff;
        uint64_t b_hi = b >> 32;
        uint64_t c_lo = c & 0xffffffff;
        uint64_t c_hi = c_lo >> 32;
        /*
                   b_hi        b_lo
                x  c_hi        c_lo
                ----------------------
                   b_hi*c_lo   c_lo*b_lo
        c_hi*b_hi  c_hi*b_lo
        */
        uint64_t lo_lo = c_lo * b_lo;
        uint64_t lo_hi = c_lo * b_hi;
        uint64_t hi_lo = c_hi * b_lo;
        uint64_t hi_hi = c_hi * b_hi;
        uint64_t mid = lo_hi + hi_lo + (lo_lo >> 32);
        uint64_t high = hi_hi + (mid >> 32);

        if ((int64_t)b < 0) high -= c;

        write_operand(rd, high);
        break;
    }
    case Mul_H_U: {
        uint64_t b_lo = b & 0xffffffff;
        uint64_t b_hi = b >> 32;
        uint64_t c_lo = c & 0xffffffff;
        uint64_t c_hi = c_lo >> 32;

        uint64_t lo_lo = c_lo * b_lo;
        uint64_t lo_hi = c_lo * b_hi;
        uint64_t hi_lo = c_hi * b_lo;
        uint64_t hi_hi = c_hi * b_hi;
        uint64_t mid = lo_hi + hi_lo + (lo_lo >> 32);
        uint64_t high = hi_hi + (mid >> 32);

        write_operand(rd, high);
        break;
    }
    case Mul_W: write_operand(rd, (int64_t)(int32_t)(b * c)); break;
    case Or: write_operand(rd, b | c); break;
    case Rem: write_operand(rd, (int64_t)b % (int64_t)c); break;
    case Rem_U: write_operand(rd, b % c); break;
    case Rem_U_W:
        write_operand(rd, (int64_t)(int32_t)((uint32_t)b % (uint32_t)c));
        break;
    case Rem_W: write_operand(rd, (int64_t)((int32_t)b % (int32_t)c)); break;
    case Store_B: writemem8(b + c, a); break;
    case Store_Cond_D:
        t = get_physical_address(b);
        if (has_reserved && reserved_addr <= t
            && t < reserved_addr + reserved_size)
        {
            writemem64(b, c);
            write_operand(rd, 0);
        }
        else
            write_operand(rd, 1);
        has_reserved = 0;
        break;
    case Store_D: writemem64(b + c, a); break;
    case SFence_VMA: // 无需同步
        break;
    case Store_H: writemem16(b + c, a); break;
    case Store_Cond_W:
        t = get_physical_address(b);
        if (has_reserved && reserved_addr <= t
            && t < reserved_addr + reserved_size)
        {
            writemem32(b, c);
            write_operand(rd, 0);
        }
        else
            write_operand(rd, 1);
        has_reserved = 0;
        break;
    case Store_W: writemem32(b + c, a); break;
    case LShiftL: write_operand(rd, b << c); break;
    case LShiftL_W: write_operand(rd, (int64_t)(int32_t)(b << c)); break;
    case SetLt: write_operand(rd, (int64_t)b < (int64_t)c); break;
    case SetLt_U: write_operand(rd, b < c); break;
    case RShiftA: write_operand(rd, (int64_t)b >> c); break;
    case RShiftA_W: write_operand(rd, (int64_t)((int32_t)b >> c)); break;
    case SRet:
        pc = read_csr(CSR_SEPC);
        CSR_SET_FIELD(SSTATUS, SIE, CSR_GET_FIELD(SSTATUS, SPIE));
        privilege_mode = CSR_GET_FIELD(SSTATUS, SPP);
        break;
    case RShiftL: write_operand(rd, b >> c); break;
    case RShiftL_W:
        write_operand(rd, (int64_t)(int32_t)((uint32_t)b >> c));
        break;
    case Sub: write_operand(rd, b - c); break;
    case Sub_W: write_operand(rd, (uint32_t)b - (uint32_t)c); break;
    case WaitInt: cpu_state = WaitingInterrupt; break;
    case Xor: write_operand(rd, b ^ c); break;
    case Invalid: longjmp(exception_jmp_env, IllegalInstruction + 1); break;
    }
}