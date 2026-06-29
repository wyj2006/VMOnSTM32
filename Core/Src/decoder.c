#include "machine.h"

Instruction decode(uint32_t instr)
{
    Instruction instruction = {
        Invalid,
        {
            {Nothing},
            {Nothing},
            {Nothing},
            {Nothing},
        },
    };

    uint8_t op = instr & 0b1111111;
    uint8_t rd = (instr >> 7) & 0b11111;
    uint8_t funct3 = (instr >> 12) & 0b111;
    uint8_t rs1 = (instr >> 15) & 0b11111;
    uint8_t rs2 = (instr >> 20) & 0b11111;
    uint8_t funct7 = (instr >> 25) & 0b1111111;
    uint64_t imm = (((int64_t)(instr >> 20) & 0xfff) << 52) >> 52;

    switch (op)
    {
    case 0b0000011:
        instruction.operands[0].kind = IntReg;
        instruction.operands[0].reg_index = rd;
        instruction.operands[1].kind = IntReg;
        instruction.operands[1].reg_index = rs1;
        instruction.operands[2].kind = Immediate;
        instruction.operands[2].imm = imm;
        switch (funct3)
        {
        case 0b000: instruction.opcode = Load_B; break;
        case 0b001: instruction.opcode = Load_H; break;
        case 0b010: instruction.opcode = Load_W; break;
        case 0b011: instruction.opcode = Load_D; break;
        case 0b100: instruction.opcode = Load_BU; break;
        case 0b101: instruction.opcode = Load_HU; break;
        case 0b110: instruction.opcode = Load_WU; break;
        }
        break;
    case 0b0000111:
        instruction.operands[0].kind = FPReg;
        instruction.operands[0].reg_index = rd;
        instruction.operands[1].kind = IntReg;
        instruction.operands[1].reg_index = rs1;
        instruction.operands[2].kind = Immediate;
        instruction.operands[2].imm = imm;
        switch (funct3)
        {
        case 0b010: instruction.opcode = FLoad_W; break;
        case 0b011: instruction.opcode = FLoad_D; break;
        }
        break;
    case 0b0001111:
        instruction.operands[0].kind = Immediate;
        instruction.operands[0].imm = (instr >> 24) & 0xf;
        instruction.operands[1].kind = Immediate;
        instruction.operands[1].imm = (instr >> 20) & 0xf;
        if (instr == 0x100f)
            instruction.opcode = FenceI;
        else
            instruction.opcode = Fence;
        break;
    case 0b0010011:
        instruction.operands[0].kind = IntReg;
        instruction.operands[0].reg_index = rd;
        instruction.operands[1].kind = IntReg;
        instruction.operands[1].reg_index = rs1;
        instruction.operands[2].kind = Immediate;
        instruction.operands[2].imm = imm;
        switch (funct3)
        {
        case 0b000: instruction.opcode = Add; break;
        case 0b001: instruction.opcode = LShiftL; break;
        case 0b010: instruction.opcode = SetLt; break;
        case 0b101:
            switch (funct7 >> 1)
            {
            case 0b000000: instruction.opcode = RShiftL; break;
            case 0b010000: instruction.opcode = RShiftA; break;
            }

            break;
        case 0b011: instruction.opcode = SetLt_U; break;
        case 0b100: instruction.opcode = Xor; break;
        case 0b110: instruction.opcode = Or; break;
        case 0b111: instruction.opcode = And; break;
        }
        break;
    case 0b0010111:
        instruction.operands[0].kind = IntReg;
        instruction.operands[0].reg_index = rd;
        instruction.operands[1].kind = Immediate;
        instruction.operands[1].imm =
            (((int64_t)((instr >> 12) & 0xfffff)) << 44) >> 32;
        instruction.opcode = AUIPC;
        break;
    case 0b0011011:
        instruction.operands[0].kind = IntReg;
        instruction.operands[0].reg_index = rd;
        instruction.operands[1].kind = IntReg;
        instruction.operands[1].reg_index = rs1;
        instruction.operands[2].kind = Immediate;
        instruction.operands[2].imm = imm;
        switch (funct3)
        {
        case 0b000: instruction.opcode = Add_W; break;
        case 0b001: instruction.opcode = LShiftL_W; break;
        case 0b101:
            switch (funct7 >> 1)
            {
            case 0b000000: instruction.opcode = RShiftL_W; break;
            case 0b010000: instruction.opcode = RShiftA_W; break;
            }
            break;
        }
        break;
    case 0b0100011:
        instruction.operands[0].kind = IntReg;
        instruction.operands[0].reg_index = rs2;
        instruction.operands[1].kind = IntReg;
        instruction.operands[1].reg_index = rs1;
        instruction.operands[2].kind = Immediate;
        instruction.operands[2].imm =
            (((int64_t)((((instr >> 25) & 0b1111111) << 5)
                        | ((instr >> 7) & 0b11111)))
             << 52)
            >> 52;
        switch (funct3)
        {
        case 0b000: instruction.opcode = Store_B; break;
        case 0b001: instruction.opcode = Store_H; break;
        case 0b010: instruction.opcode = Store_W; break;
        case 0b011: instruction.opcode = Store_D; break;
        }
        break;
    case 0b0100111:
        instruction.operands[0].kind = FPReg;
        instruction.operands[0].reg_index = rs2;
        instruction.operands[1].kind = IntReg;
        instruction.operands[1].reg_index = rs1;
        instruction.operands[2].kind = Immediate;
        instruction.operands[2].imm =
            (((int64_t)((((instr >> 25) & 0b1111111) << 5)
                        | ((instr >> 7) & 0b11111)))
             << 52)
            >> 52;
        switch (funct3)
        {
        case 0b010: instruction.opcode = FStore_W; break;
        case 0b011: instruction.opcode = FStore_D; break;
        }
        break;
    case 0b0101111:
        instruction.operands[0].kind = IntReg;
        instruction.operands[0].reg_index = rd;
        instruction.operands[1].kind = IntReg;
        instruction.operands[1].reg_index = rs1;
        instruction.operands[2].kind = IntReg;
        instruction.operands[2].reg_index = rs2;
        switch (funct3)
        {
        case 0b010:
            switch (rs2)
            {
            case 0b00000:
                switch (funct7 >> 2)
                {
                case 0b00010: instruction.opcode = Load_Reserved_W; break;
                }
                break;
            default:
                switch (funct7 >> 2)
                {
                case 0b00000: instruction.opcode = AMO_Add_W; break;
                case 0b00001: instruction.opcode = AMO_Swap_W; break;
                case 0b00011: instruction.opcode = Store_Cond_W; break;
                case 0b00100: instruction.opcode = AMO_Xor_W; break;
                case 0b01000: instruction.opcode = AMO_Or_W; break;
                case 0b01100: instruction.opcode = AMO_And_W; break;
                case 0b10000: instruction.opcode = AMO_Min_W; break;
                case 0b10100: instruction.opcode = AMO_Max_W; break;
                case 0b11000: instruction.opcode = AMO_Min_UW; break;
                case 0b11100: instruction.opcode = AMO_Max_UW; break;
                }
            }

            break;
        case 0b011:
            switch (rs2)
            {
            case 0b00000:
                switch (funct7 >> 2)
                {
                case 0b00010: instruction.opcode = Load_Reserved_D; break;
                }
                break;
            default:
                switch (funct7 >> 2)
                {
                case 0b00000: instruction.opcode = AMO_Add_D; break;
                case 0b00001: instruction.opcode = AMO_Swap_D; break;
                case 0b00011: instruction.opcode = Store_Cond_D; break;
                case 0b00100: instruction.opcode = AMO_Xor_D; break;
                case 0b01000: instruction.opcode = AMO_Or_D; break;
                case 0b01100: instruction.opcode = AMO_And_D; break;
                case 0b10000: instruction.opcode = AMO_Min_D; break;
                case 0b10100: instruction.opcode = AMO_Max_D; break;
                case 0b11000: instruction.opcode = AMO_Min_UD; break;
                case 0b11100: instruction.opcode = AMO_Max_UD; break;
                }
            }
            break;
        }
        break;
    case 0b0110011:
        instruction.operands[0].kind = IntReg;
        instruction.operands[0].reg_index = rd;
        instruction.operands[1].kind = IntReg;
        instruction.operands[1].reg_index = rs1;
        instruction.operands[2].kind = IntReg;
        instruction.operands[2].reg_index = rs2;
        switch (funct3)
        {
        case 0b000:
            switch (funct7)
            {
            case 0b0000000: instruction.opcode = Add; break;
            case 0b0000001: instruction.opcode = Mul; break;
            case 0b0100000: instruction.opcode = Sub; break;
            }
            break;
        case 0b001:
            switch (funct7)
            {
            case 0b0000000: instruction.opcode = LShiftL; break;
            case 0b0000001: instruction.opcode = Mul_H; break;
            }
            break;
        case 0b010:
            switch (funct7)
            {
            case 0b0000000: instruction.opcode = SetLt; break;
            case 0b000001: instruction.opcode = Mul_H_SU; break;
            }
            break;
        case 0b011:
            switch (funct7)
            {
            case 0b0000000: instruction.opcode = SetLt_U; break;
            case 0b000001: instruction.opcode = Mul_H_U; break;
            }
            break;
        case 0b100:
            switch (funct7)
            {
            case 0b0000000: instruction.opcode = Xor; break;
            case 0b0000001: instruction.opcode = Div; break;
            }
            break;
        case 0b101:
            switch (funct7)
            {
            case 0b0000000: instruction.opcode = RShiftL; break;
            case 0b0000001: instruction.opcode = Div_U; break;
            case 0b0100000: instruction.opcode = RShiftA; break;
            }
            break;
        case 0b110:
            switch (funct7)
            {
            case 0b0000000: instruction.opcode = Or; break;
            case 0b0000001: instruction.opcode = Rem; break;
            }
            break;
        case 0b111:
            switch (funct7)
            {
            case 0b0000000: instruction.opcode = And; break;
            case 0b0000001: instruction.opcode = Rem_U; break;
            }
        }
        break;
    case 0b0110111:
        instruction.operands[0].kind = IntReg;
        instruction.operands[0].reg_index = rd;
        instruction.operands[1].kind = Immediate;
        instruction.operands[1].imm = ((int64_t)(instr & ~0xfff) << 32) >> 32;
        instruction.opcode = Load_Imm;
        break;
    case 0b0111011:
        instruction.operands[0].kind = IntReg;
        instruction.operands[0].reg_index = rd;
        instruction.operands[1].kind = IntReg;
        instruction.operands[1].reg_index = rs1;
        instruction.operands[2].kind = IntReg;
        instruction.operands[2].reg_index = rs2;
        switch (funct3)
        {
        case 0b000:
            switch (funct7)
            {
            case 0b0000000: instruction.opcode = Add_W; break;
            case 0b0000001: instruction.opcode = Mul_W; break;
            case 0b0100000: instruction.opcode = Sub_W; break;
            }
            break;
        case 0b001:
            switch (funct7)
            {
            case 0b0000000: instruction.opcode = LShiftL_W; break;
            }
            break;
        case 0b100:
            switch (funct7)
            {
            case 0b0000001: instruction.opcode = Div_W; break;
            }
            break;
        case 0b101:
            switch (funct7)
            {
            case 0b0000000: instruction.opcode = RShiftL_W; break;
            case 0b0000001: instruction.opcode = Div_UW; break;
            case 0b0100000: instruction.opcode = RShiftA_W; break;
            }
            break;
        case 0b110:
            switch (funct7)
            {
            case 0b0000001: instruction.opcode = Rem_W; break;
            }
            break;
        case 0b111:
            switch (funct7)
            {
            case 0b0000001: instruction.opcode = Rem_U_W; break;
            }
            break;
        }
        break;
    case 0b1000011:
        instruction.operands[0].kind = FPReg;
        instruction.operands[0].reg_index = rd;
        instruction.operands[1].kind = FPReg;
        instruction.operands[1].reg_index = rs1;
        instruction.operands[2].kind = FPReg;
        instruction.operands[2].reg_index = rs2;
        instruction.operands[3].kind = FPReg;
        instruction.operands[3].reg_index = (instr >> 27) & 0b11111;
        switch (funct3)
        {
        default:
            switch (funct7 & 0b11)
            {
            case 0b00: instruction.opcode = FMulAdd_S; break;
            case 0b01: instruction.opcode = FMulAdd_D; break;
            }
        }
        break;
    case 0b1000111:
        instruction.operands[0].kind = FPReg;
        instruction.operands[0].reg_index = rd;
        instruction.operands[1].kind = FPReg;
        instruction.operands[1].reg_index = rs1;
        instruction.operands[2].kind = FPReg;
        instruction.operands[2].reg_index = rs2;
        instruction.operands[3].kind = FPReg;
        instruction.operands[3].reg_index = (instr >> 27) & 0b11111;
        switch (funct3)
        {
        default:
            switch (funct7 & 0b11)
            {
            case 0b00: instruction.opcode = FMulSub_S; break;
            case 0b01: instruction.opcode = FMulSub_D; break;
            }
        }
        break;
    case 0b1001011:
        instruction.operands[0].kind = FPReg;
        instruction.operands[0].reg_index = rd;
        instruction.operands[1].kind = FPReg;
        instruction.operands[1].reg_index = rs1;
        instruction.operands[2].kind = IntReg;
        instruction.operands[2].reg_index = rs2;
        instruction.operands[3].kind = IntReg;
        instruction.operands[3].reg_index = (instr >> 27) & 0b11111;
        switch (funct7 & 0b11)
        {
        case 0b00: instruction.opcode = FNegMulSub_S; break;
        case 0b01: instruction.opcode = FNegMulSub_D; break;
        }
        break;
    case 0b1001111:
        instruction.operands[0].kind = FPReg;
        instruction.operands[0].reg_index = rd;
        instruction.operands[1].kind = FPReg;
        instruction.operands[1].reg_index = rs1;
        instruction.operands[2].kind = FPReg;
        instruction.operands[2].reg_index = rs2;
        instruction.operands[3].kind = FPReg;
        instruction.operands[3].reg_index = (instr >> 27) & 0b11111;
        switch (funct7 & 0b11)
        {
        case 0b00: instruction.opcode = FNegMulAdd_S; break;
        case 0b01: instruction.opcode = FNegMulAdd_D; break;
        }
        break;
    case 0b1010011:
        instruction.operands[0].kind = FPReg;
        instruction.operands[0].reg_index = rd;
        instruction.operands[1].kind = FPReg;
        instruction.operands[1].reg_index = rs1;
        instruction.operands[2].kind = FPReg;
        instruction.operands[2].reg_index = rs2;
        switch (funct3)
        {
        case 0b000:
            switch (rs2)
            {
            case 0b00000:
                switch (funct7)
                {
                case 0b1110000:
                    instruction.operands[0].kind = IntReg;
                    instruction.opcode = FMove_X_W;
                    break;
                case 0b1110001:
                    instruction.operands[0].kind = IntReg;
                    instruction.opcode = FMove_X_D;
                    break;
                case 0b1111000:
                    instruction.operands[1].kind = IntReg;
                    instruction.opcode = FMove_S_X;
                    break;
                case 0b1111001:
                    instruction.operands[1].kind = IntReg;
                    instruction.opcode = FMove_D_X;
                    break;
                }
                break;
            default:
                switch (funct7)
                {
                case 0b0010000: instruction.opcode = FSgnJ_S; break;
                case 0b0010001: instruction.opcode = FSgnJ_D; break;
                case 0b0010100: instruction.opcode = FMin_S; break;
                case 0b0010101: instruction.opcode = FMin_D; break;
                case 0b1010000:
                    instruction.operands[0].kind = IntReg;
                    instruction.opcode = FLe_S;
                    break;
                case 0b1010001:
                    instruction.operands[0].kind = IntReg;
                    instruction.opcode = FLe_D;
                    break;
                }
            }

            break;
        case 0b001:
            switch (rs2)
            {
            case 0b00000:
                switch (funct7)
                {
                case 0b0010100: instruction.opcode = FMax_S; break;
                case 0b0010101: instruction.opcode = FMax_D; break;
                case 0b1010000:
                    instruction.operands[0].kind = IntReg;
                    instruction.opcode = FLt_S;
                    break;
                case 0b1010001:
                    instruction.operands[0].kind = IntReg;
                    instruction.opcode = FLt_D;
                    break;
                case 0b1110000:
                    instruction.operands[0].kind = IntReg;
                    instruction.opcode = FClass_S;
                    break;
                case 0b1110001:
                    instruction.operands[0].kind = IntReg;
                    instruction.opcode = FClass_D;
                    break;
                }
                break;
            default:
                switch (funct7)
                {
                case 0b0010000: instruction.opcode = FSgnJ_Neg_S; break;
                case 0b0010001: instruction.opcode = FSgnJ_Neg_D; break;
                }
            }
            break;
        case 0b010:
            switch (funct7)
            {
            case 0b0010000: instruction.opcode = FSgnJ_Xor_S; break;
            case 0b0010001: instruction.opcode = FSgnJ_Xor_D; break;
            case 0b1010000:
                instruction.operands[0].kind = IntReg;
                instruction.opcode = FEq_S;
                break;
            case 0b1010001:
                instruction.operands[0].kind = IntReg;
                instruction.opcode = FEq_D;
                break;
            }
            break;
        default:
            switch (rs2)
            {
            case 0b00000:
                switch (funct7)
                {
                case 0b0100001: instruction.opcode = FCvt_D_S; break;
                case 0b0101100: instruction.opcode = FSqrt_S; break;
                case 0b0101101: instruction.opcode = FSqrt_D; break;
                case 0b1100000:
                    instruction.operands[0].kind = IntReg;
                    instruction.opcode = FCvt_W_S;
                    break;
                case 0b1100001:
                    instruction.operands[0].kind = IntReg;
                    instruction.opcode = FCvt_W_D;
                    break;
                case 0b1101000:
                    instruction.operands[1].kind = IntReg;
                    instruction.opcode = FCvt_S_W;
                    break;
                case 0b1101001:
                    instruction.operands[1].kind = IntReg;
                    instruction.opcode = FCvt_D_W;
                    break;
                }
                break;
            case 0b00001:
                switch (funct7)
                {
                case 0b0100000: instruction.opcode = FCvt_S_D; break;
                case 0b1100000:
                    instruction.operands[0].kind = IntReg;
                    instruction.opcode = FCvt_WU_S;
                    break;
                case 0b1100001:
                    instruction.operands[0].kind = IntReg;
                    instruction.opcode = FCvt_WU_D;
                    break;
                case 0b1101000:
                    instruction.operands[1].kind = IntReg;
                    instruction.opcode = FCvt_S_WU;
                    break;
                case 0b1101001:
                    instruction.operands[1].kind = IntReg;
                    instruction.opcode = FCvt_D_WU;
                    break;
                }
                break;
            case 0b00010:
                switch (funct7)
                {
                case 0b1100000:
                    instruction.operands[0].kind = IntReg;
                    instruction.opcode = FCvt_L_S;
                    break;
                case 0b1100001:
                    instruction.operands[0].kind = IntReg;
                    instruction.opcode = FCvt_L_D;
                    break;
                case 0b1101000:
                    instruction.operands[1].kind = IntReg;
                    instruction.opcode = FCvt_S_L;
                    break;
                case 0b1101001:
                    instruction.operands[1].kind = IntReg;
                    instruction.opcode = FCvt_D_L;
                    break;
                }
                break;
            case 0b00011:
                switch (funct7)
                {
                case 0b1100000:
                    instruction.operands[0].kind = IntReg;
                    instruction.opcode = FCvt_LU_S;
                    break;
                case 0b1100001:
                    instruction.operands[0].kind = IntReg;
                    instruction.opcode = FCvt_LU_D;
                    break;
                case 0b1101000:
                    instruction.operands[1].kind = IntReg;
                    instruction.opcode = FCvt_S_LU;
                    break;
                case 0b1101001:
                    instruction.operands[1].kind = IntReg;
                    instruction.opcode = FCvt_D_LU;
                    break;
                }
                break;
            default:
                switch (funct7)
                {
                case 0b0000000: instruction.opcode = FAdd_S; break;
                case 0b0000001: instruction.opcode = FAdd_D; break;
                case 0b0000100: instruction.opcode = FSub_S; break;
                case 0b0000101: instruction.opcode = FSub_D; break;
                case 0b0001000: instruction.opcode = FMul_S; break;
                case 0b0001001: instruction.opcode = FMul_D; break;
                case 0b0001100: instruction.opcode = FDiv_S; break;
                case 0b0001101: instruction.opcode = FDiv_D; break;
                }
            }
        }
        break;
    case 0b1100011:
        instruction.operands[0].kind = IntReg;
        instruction.operands[0].reg_index = rs1;
        instruction.operands[1].kind = IntReg;
        instruction.operands[1].reg_index = rs2;
        instruction.operands[2].kind = Immediate;
        instruction.operands[2].imm =
            (((int64_t)((((instr >> 31) & 1) << 12) | (((instr >> 7) & 1) << 11)
                        | (((instr >> 25) & 0b111111) << 5)
                        | (((instr >> 8) & 0b1111) << 1)))
             << 52)
            >> 52;
        switch (funct3)
        {
        case 0b000: instruction.opcode = BEq; break;
        case 0b001: instruction.opcode = BNe; break;
        case 0b100: instruction.opcode = BLt; break;
        case 0b101: instruction.opcode = BGe; break;
        case 0b110: instruction.opcode = BLt_U; break;
        case 0b111: instruction.opcode = BGe_U; break;
        }
        break;
    case 0b1100111:
        instruction.operands[0].kind = IntReg;
        instruction.operands[0].reg_index = rd;
        instruction.operands[1].kind = IntReg;
        instruction.operands[1].reg_index = rs1;
        instruction.operands[2].kind = Immediate;
        instruction.operands[2].imm = imm;
        switch (funct3)
        {
        case 0b010: instruction.opcode = JAL_Reg; break;
        }
        break;
    case 0b1101111:
        instruction.operands[0].kind = IntReg;
        instruction.operands[0].reg_index = rd;
        instruction.operands[1].kind = Immediate;
        instruction.operands[1].imm =
            (((int64_t)(((instr >> 31) << 20) | (((instr >> 12) & 0xff) << 12)
                        | (((instr >> 20) & 1) << 11)
                        | (((instr >> 21) & 0x3ff) << 1)))
             << 43)
            >> 43;
        instruction.opcode = JAL;
        break;
    case 0b1110011:
        switch (instr)
        {
        case 0x00000073: instruction.opcode = ECall; break;
        case 0x00100073: instruction.opcode = EBreak; break;
        case 0x10200073: instruction.opcode = SRet; break;
        case 0x10500073: instruction.opcode = WaitInt; break;
        case 0x30200073: instruction.opcode = MRet; break;
        default:
            if (rd == 0 && funct3 == 0 && funct7 == 0b0001001)
            {
                instruction.operands[0].kind = IntReg;
                instruction.operands[0].reg_index = rs1;
                instruction.operands[1].kind = IntReg;
                instruction.operands[1].reg_index = rs2;
                instruction.opcode = SFence_VMA;
            }
            else
            {
                instruction.operands[0].kind = IntReg;
                instruction.operands[0].reg_index = rd;
                instruction.operands[1].kind = CSR;
                instruction.operands[1].csr_index = (instr >> 20) & 0xfff;
                if (((funct3 >> 2) & 1) == 0)
                {
                    instruction.operands[2].kind = IntReg;
                    instruction.operands[2].reg_index = rs1;
                }
                else
                {
                    instruction.operands[2].kind = Immediate;
                    instruction.operands[2].reg_index = imm;
                }
                switch (funct3 & 0b11)
                {
                case 0b01: instruction.opcode = CSR_RW; break;
                case 0b10: instruction.opcode = CSR_RS; break;
                case 0b11: instruction.opcode = CSR_RC; break;
                }
            }
        }

        break;
    }

    return instruction;
}