.section ".text.boot"

.global _start
.option norvc

/* BROM header */
_start:
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

/* entry point */
_payload:

/* disable interrupts */
csrw mie, zero

/* enable THEAD extended instruction set */
li t1, 1<<22
csrs 0x7c0, t1

/* invalidate caches (MCOR CSR) */
li t2, 0x30013
csrs 0x7c2, t2

/* setup stack at the top of SRAM A1 */
li sp, 0x00027FF0

/* zero out bss */
ld t0, __bss_start
ld t1, __bss_end

_zero_bss:
beq t0, t1, _boot_main
sw zero, 0(t0)
addi t0, t0, 4
j _zero_bss

_boot_main:
/* jump to rust code */
j _main

_hang:
j _hang
