#include <string.h>

#include "usart.h"

#include "machine.h"
#include "privilege.h"

#define READ_MEM_CODE 0
#define WRITE_MEM_CODE 1
#define FRAME_START 0xaa
#define FRAME_END 0x55

__attribute__((section(".ram"))) uint8_t memory[INTERNAL_MEM_SIZE];
uint8_t is_load;
uint64_t reserved_size;
uint8_t has_reserved;
uint64_t reserved_addr;

void send_frame(uint8_t frame[], uint64_t frame_size)
{
    HAL_UART_Transmit(&huart2, &(uint8_t){FRAME_START}, 1, HAL_MAX_DELAY);
    for (int i = 0; i < frame_size; i++)
    {
        if (frame[i] == FRAME_START || frame[i] == FRAME_END)
            HAL_UART_Transmit(&huart2, &(uint8_t){'\\'}, 1, HAL_MAX_DELAY);
        HAL_UART_Transmit(&huart2, frame + i, 1, HAL_MAX_DELAY);
    }
    HAL_UART_Transmit(&huart2, &(uint8_t){FRAME_END}, 1, HAL_MAX_DELAY);
}

uint8_t raw_readmem8(uint64_t address)
{
    if (address > EXTERNAL_MEM_SIZE) goto access_fault;
    if (address < INTERNAL_MEM_SIZE) return memory[address];

    uint8_t frame[9];
    frame[0] = READ_MEM_CODE;
    memcpy(frame + 1, &address, sizeof(address));
    send_frame(frame, sizeof(frame));

    uint8_t data;
    HAL_UART_Receive(&huart2, &data, sizeof(data), HAL_MAX_DELAY);
    return data;

access_fault:
    longjmp(exception_jmp_env, LoadAccessFault);
    return 0;
}

void raw_writemem8(uint64_t address, uint8_t value)
{
    if (address > EXTERNAL_MEM_SIZE) goto access_fault;

    if (reserved_addr <= address && address < reserved_addr + reserved_size)
        has_reserved = 0;

    if (address < INTERNAL_MEM_SIZE)
    {
        memory[address] = value;
        return;
    }

    uint8_t frame[10];
    frame[0] = WRITE_MEM_CODE;
    memcpy(frame + 1, &address, sizeof(address));
    memcpy(frame + 1 + sizeof(address), &value, sizeof(value));
    send_frame(frame, sizeof(frame));

    HAL_UART_Transmit(&huart2, frame, sizeof(address), HAL_MAX_DELAY);

    return;
access_fault:
    longjmp(exception_jmp_env, LoadAccessFault);
}

uint64_t get_physical_address(uint64_t address)
{
    if (privilege_mode == Machine) return address;

    uint64_t satp = read_csr(CSR_SATP);
    uint64_t ppn = satp & 0xfffffffffff;
    uint64_t level, sv;

    switch (satp >> 60)
    {
    case 0: return address;
    case 8:
        level = 3;
        sv = 39;
        break;
    case 9:
        level = 4;
        sv = 48;
        break;
    case 11:
        level = 5;
        sv = 57;
        break;
    }

    uint64_t vpn_len = (sv - 12) / level;
    for (int i = 0; i < level - 1; i++)
    {
        uint64_t vpn =
            (address >> 12 >> (vpn_len * i)) & ((1ull << vpn_len) - 1);

        uint64_t pte_address = ppn * 4096 + vpn * 4;
        uint64_t pte = raw_readmem8(pte_address)
                       | (raw_readmem8(pte_address + 1) << 8)
                       | (raw_readmem8(pte_address + 2) << 16)
                       | (raw_readmem8(pte_address + 3) << 24)
                       | ((uint64_t)raw_readmem8(pte_address + 4) << 32)
                       | ((uint64_t)raw_readmem8(pte_address + 5) << 40)
                       | ((uint64_t)raw_readmem8(pte_address + 6) << 48)
                       | ((uint64_t)raw_readmem8(pte_address + 7) << 56);

        if ((pte & 1) == 0) goto page_fault;

        switch ((pte >> 4) & 1)
        {
        case 0:
            if (privilege_mode == User) goto page_fault;
            break;
        case 1:
            if (privilege_mode == Supervisor && !CSR_GET_FIELD(SSTATUS, SUM))
                goto page_fault;
            break;
        }

        ppn = (pte >> 10) & 0xfffffffffff;
    }

    return ppn * 4096 + address & 0xfff;

page_fault:
    if (is_load)
        longjmp(exception_jmp_env, LoadPageFault);
    else if (is_load)
        longjmp(exception_jmp_env, StorePageFault);
    return 0;
}

uint8_t readmem8(uint64_t adderss)
{
    is_load = 1;
    return raw_readmem8(get_physical_address(adderss));
}

void writemem8(uint64_t address, uint8_t value)
{
    is_load = 0;
    raw_writemem8(get_physical_address(address), value);
}

uint16_t readmem16(uint64_t address)
{
    return readmem8(address) | (readmem8(address + 1) << 8);
}

void writemem16(uint64_t address, uint16_t value)
{
    writemem8(address, value);
    writemem8(address + 1, value >> 8);
}

uint32_t readmem32(uint64_t address)
{
    return readmem8(address) | (readmem8(address + 1) << 8)
           | (readmem8(address + 2) << 16) | (readmem8(address + 3) << 24);
}

void writemem32(uint64_t address, uint32_t value)
{
    writemem8(address, value);
    writemem8(address + 1, value >> 8);
    writemem8(address + 2, value >> 16);
    writemem8(address + 3, value >> 24);
}

uint64_t readmem64(uint64_t address)
{
    return readmem8(address) | (readmem8(address + 1) << 8)
           | (readmem8(address + 2) << 16) | (readmem8(address + 3) << 24)
           | ((uint64_t)readmem8(address + 4) << 32)
           | ((uint64_t)readmem8(address + 5) << 40)
           | ((uint64_t)readmem8(address + 6) << 48)
           | ((uint64_t)readmem8(address + 7) << 56);
}

void writemem64(uint64_t address, uint64_t value)
{
    writemem8(address, value);
    writemem8(address + 1, value >> 8);
    writemem8(address + 2, value >> 16);
    writemem8(address + 3, value >> 24);
    writemem8(address + 4, value >> 32);
    writemem8(address + 5, value >> 40);
    writemem8(address + 6, value >> 48);
    writemem8(address + 7, value >> 56);
}