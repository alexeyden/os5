.text

.global _start

j _payload        /* jump over the metadata below to the actual payload */
.ascii "eGON.BT0" /* header marker (magic) */
.word 0x5f0a6c39  /* checksum initial value */
.word 0x00000000  /* payload size */
.word _payload - _start /* header size */
.word 0 /* public header size (we don't need one) */
.word 0 /* public header version */
.word 0 /* return address (dont care about this one) */
.word 0x20000 /* run address of the payload (SRAM A1) */
.word 0 /* boot cpu / eGON version (don't care) */
.dword 0 /* platform information (don't care) */

_payload:
.equ GPIO_BASE, 0x02000000
.equ GPIO_PD_CFG2, 0x98
.equ GPIO_PD_DAT, 0xA0

/* Configure PD18 as output: GPIO_PD_CFG2[11:8] = 0b0001 */
li a0, GPIO_BASE + GPIO_PD_CFG2
lw a1, 0(a0)
li a2, 0xfffff0ff
and a1, a1, a2
ori a1, a1, 0x100
sw a1, 0(a0)

/* Invert PD18 state: ~GPIO_PD_DAT[18] */
li a0, GPIO_BASE + GPIO_PD_DAT
lw a1, 0(a0)
li a2, 1<<18

_blink:
xor a1, a1, a2
sw a1, 0(a0)

/* Sleep for 500ms and jump back */
li a3, 0
li a4,  0x40d9000
_delay:
addi a3, a3, 1
beq a3, a4, _blink
j _delay

_hang:
j _hang
