.option norvc
.section .boot, "ax",@progbits
.global _start

_start:
    lui     sp, %hi(stacks + 1024)
    ori     sp, sp, %lo(stacks + 1024)
    j       __start_rust

.bss

.global stacks
stacks:
    .skip 1024
