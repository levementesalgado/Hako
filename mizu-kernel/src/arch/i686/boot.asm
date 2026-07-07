.intel_syntax noprefix

.section .multiboot
.align 4
.long 0x1BADB002
.long 0x03
.long -(0x1BADB002 + 0x03)

.section .text
.globl start
.extern kmain

start:
    mov esp, offset stack_top
    push ebx
    push eax
    call kmain
    cli
1:  hlt
    jmp 1b

.section .bss
.align 16
stack_bottom:
    .space 16384
stack_top:
