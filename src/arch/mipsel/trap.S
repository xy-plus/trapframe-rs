# 1 "src/arch/mipsel/boot/trap.S"
# 1 "<built-in>" 1
# 1 "src/arch/mipsel/boot/trap.S" 2
# 1 "src/arch/mipsel/boot/regdef.h" 1
# 2 "src/arch/mipsel/boot/trap.S" 2

 .set noat
 .set noreorder
    .section .text.ebase
    .globl trap_entry

.org 0x0
trap_entry:
 # +0x000: TLB-miss vector
 b general_trap_vec

 # +0x180: general vector
.org 0x180
general_trap_vec:
 move $27, $29 # save stack pointer to $27
 mfc0 $26, $12 # read cp0.status
 andi $26, $26, 0x10 # extract cp0.status.ksu
 beq $26, $0, trap_from_kernel
 nop # delayslot

trap_from_user:
 # load kstack, we can use $26 to store something
 # la $26, kernel_stack
 # la $29, kernel_stack_top
 la $26, _cur_kstack_ptr
 lw $29, 0($26)

trap_from_kernel:







 # align stack pointer
 andi $26, $29, 0xf
 beqz $26, run_user
 nop

 la $26, 0xfffffff0
 and $26, $29, $26
 sw $29, -176($26)
 move $29, $26

 .globl run_user
run_user:
 # allocate 38 / 70 words for trapframe + 6 extra words

 addiu $29, $29, -176

 # save general registers
 sw $31, 160($29)
 sw $30, 156($29)
 sw $27, 152($29) # $27 = old $29
 sw $28, 148($29)
 sw $27, 144($29) # real $27 is damaged
 sw $26, 140($29) # real $26 is damaged
 sw $25, 136($29)
 sw $24, 132($29)
 sw $23, 128($29)
 sw $22, 124($29)
 sw $21, 120($29)
 sw $20, 116($29)
 sw $19, 112($29)
 sw $18, 108($29)
 sw $17, 104($29)
 sw $16, 100($29)
 sw $15, 96($29)
 sw $14, 92($29)
 sw $13, 88($29)
 sw $12, 84($29)
 sw $11, 80($29)
 sw $10, 76($29)
 sw $9, 72($29)
 sw $8, 68($29)
 sw $7, 64($29)
 sw $6, 60($29)
 sw $5, 56($29)
 sw $4, 52($29)
 sw $3, 48($29)
 sw $2, 44($29)
 sw $1, 40($29)
 nop

 # save hi/lo
 mflo $9
 sw $9, 36($29)
 mfhi $8
 sw $8, 32($29)

 # save special registers
 mfc0 $8, $8 # cp0.vaddr
 sw $8, 28($29)

 mfc0 $9, $14 # cp0.epc
 sw $9, 24($29)

 mfc0 $8, $13 # cp0.cause
 sw $8, 20($29)

 mfc0 $9, $12 # cp0.status
 sw $9, 16($29)

 # support nested interrupt
 la $8, ~0x1b # reset status.ksu, status.exl, status.ie
 and $9, $9, $8
 mtc0 $9, $12 # cp0.status

 # prepare to call rust_trap
 ori $4, $29, 0
 jal rust_trap
 nop

 .globl trap_return
trap_return:
 # restore special registers
 lw $9, 16($29)
 ori $9, $9, 0x2 # status.exl
 nop
 mtc0 $9, $12 # cp0.status

 lw $26, 24($29)
 mtc0 $26, $14 # cp0.epc

 lw $8, 32($29)
 mthi $8
 lw $9, 36($29)
 mtlo $9

 # restore general registers
 lw $1, 40($29)
 lw $2, 44($29)
 lw $3, 48($29)
 lw $4, 52($29)
 lw $5, 56($29)
 lw $6, 60($29)
 lw $7, 64($29)
 lw $8, 68($29)
 lw $9, 72($29)
 lw $10, 76($29)
 lw $11, 80($29)
 lw $12, 84($29)
 lw $13, 88($29)
 lw $14, 92($29)
 lw $15, 96($29)
 lw $16, 100($29)
 lw $17, 104($29)
 lw $18, 108($29)
 lw $19, 112($29)
 lw $20, 116($29)
 lw $21, 120($29)
 lw $22, 124($29)
 lw $23, 128($29)
 lw $24, 132($29)
 lw $25, 136($29)

 # lw $26, 140($29)
 # lw $27, 144($29)
 lw $28, 148($29)
 lw $30, 156($29)
 lw $31, 160($29)

 # save kernel stack
 lw $26, 0($29)
 addiu $27, $29, 176
 movn $27, $26, $26

 la $26, _cur_kstack_ptr
 sw $27, 0($26)
 nop


 lw $29, 152($29)

 eret
 nop

    .section .bss.stack
    .align 12 #PGSHIFT
    .global kernel_stack
kernel_stack:
    .space 1024 * 16 # 16KB for kernel stack
    .global kernel_stack_top
kernel_stack_top:

    .align 12 #PGSHIFT
    .global _root_page_table_buffer
_root_page_table_buffer:
    .space 1024 * 64 # 64KB
    .global _root_page_table_ptr
_root_page_table_ptr:
    .space 4 # 4bytes
    .global _cur_kstack_ptr
_cur_kstack_ptr:
    .space 4 # 4bytes
    .global _cur_tls
_cur_tls:
    .space 4 # 4bytes
