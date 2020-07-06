use mips::registers::cp0;

global_asm!(include_str!("trap.S"));

#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct GeneralRegs {
    pub zero: usize, /* wired zero */
    pub at: usize,   /* assembler temp  - uppercase because of ".set at" */
    pub v0: usize,   /* return value */
    pub v1: usize,
    pub a0: usize, /* argument registers */
    pub a1: usize,
    pub a2: usize,
    pub a3: usize,
    pub t0: usize, /* caller saved */
    pub t1: usize,
    pub t2: usize,
    pub t3: usize,
    pub t4: usize,
    pub t5: usize,
    pub t6: usize,
    pub t7: usize,
    pub s0: usize, /* callee saved */
    pub s1: usize,
    pub s2: usize,
    pub s3: usize,
    pub s4: usize,
    pub s5: usize,
    pub s6: usize,
    pub s7: usize,
    pub t8: usize, /* caller saved */
    pub t9: usize, /* same like jp? */
    // pub jp: usize, /* PIC jump register */
    pub k0: usize, /* kernel scratch */
    pub k1: usize,
    pub gp: usize, /* global pointer */
    pub sp: usize, /* stack pointer */
    pub fp: usize, /* frame pointer */
    // pub s8: usize,  /* same like fp! */
    pub ra: usize, /* return address */
}

/// Saved registers on a trap.
#[derive(Clone)]
#[repr(C)]
pub struct TrapFrame {
    /// Non-zero if the kernel stack is not 16-byte-aligned
    pub unaligned_kstack: usize,
    /// unused 12 bytes
    pub unused: [usize; 3],
    /// CP0 status register
    pub status: cp0::status::Status,
    /// CP0 cause register
    pub cause: cp0::cause::Cause,
    /// CP0 EPC register
    pub epc: usize,
    /// CP0 vaddr register
    pub vaddr: usize,
    /// HI/LO registers
    pub hi: usize,
    pub lo: usize,
    /// General registers
    pub general: GeneralRegs,
}

#[allow(dead_code)]
impl TrapFrame {
    /// Constructs TrapFrame for a new kernel thread.
    ///
    /// The new thread starts at function `entry` with an usize argument `arg`.
    /// The stack pointer will be set to `sp`.
    pub fn new_kernel_thread(entry: extern "C" fn(usize) -> !, arg: usize, sp: usize) -> Self {
        use core::mem::zeroed;
        let mut tf: Self = unsafe { zeroed() };
        tf.general.a0 = arg;
        tf.general.sp = sp;
        tf.epc = entry as usize;
        tf.status = cp0::status::read();
        tf.status.set_kernel_mode();
        tf.status.set_ie();
        tf.status.set_exl();
        tf
    }

    /// Constructs TrapFrame for a new user thread.
    ///
    /// The new thread starts at `entry_addr`.
    /// The stack pointer will be set to `sp`.
    pub fn new_user_thread(entry_addr: usize, sp: usize) -> Self {
        use core::mem::zeroed;
        let mut tf: Self = unsafe { zeroed() };
        tf.general.sp = sp;
        tf.epc = entry_addr;
        tf.status = cp0::status::read();
        tf.status.set_user_mode();
        tf.status.set_ie();
        tf.status.set_exl();
        tf
    }
}

use core::fmt::{Debug, Error, Formatter};
impl Debug for TrapFrame {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        f.debug_struct("TrapFrame")
            .field("status", &self.status.bits)
            .field("epc", &self.epc)
            .field("cause", &self.cause.bits)
            .field("vaddr", &self.vaddr)
            .field("at", &self.general.at)
            .field("v0", &self.general.v0)
            .field("v1", &self.general.v1)
            .field("a0", &self.general.a0)
            .field("a1", &self.general.a1)
            .field("a2", &self.general.a2)
            .field("a3", &self.general.a3)
            .field("t0", &self.general.t0)
            .field("t1", &self.general.t1)
            .field("t2", &self.general.t2)
            .field("t3", &self.general.t3)
            .field("t4", &self.general.t4)
            .field("t5", &self.general.t5)
            .field("t6", &self.general.t6)
            .field("t7", &self.general.t7)
            .field("s0", &self.general.s0)
            .field("s1", &self.general.s1)
            .field("s2", &self.general.s2)
            .field("s3", &self.general.s3)
            .field("s4", &self.general.s4)
            .field("s5", &self.general.s5)
            .field("s6", &self.general.s6)
            .field("s7", &self.general.s7)
            .field("t8", &self.general.t8)
            .field("t9", &self.general.t9)
            .field("k0", &self.general.k0)
            .field("k1", &self.general.k1)
            .field("gp", &self.general.gp)
            .field("sp", &self.general.sp)
            .field("fp", &self.general.fp)
            .field("ra", &self.general.ra)
            .finish()
    }
}

/// Saved registers on a trap.
#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct UserContext {
    /// Non-zero if the kernel stack is not 16-byte-aligned
    pub unaligned_kstack: usize,
    /// unused 12 bytes
    pub unused: [usize; 3],
    /// CP0 status register
    pub status: usize,
    /// CP0 cause register
    pub cause: usize,
    /// CP0 EPC register
    pub epc: usize,
    /// CP0 vaddr register
    pub vaddr: usize,
    /// HI/LO registers
    pub hi: usize,
    pub lo: usize,
    /// General registers
    pub general: GeneralRegs,
}

impl UserContext {
    /// Get number of syscall
    pub fn get_syscall_num(&self) -> usize {
        self.general.v0
    }

    /// Get return value of syscall
    pub fn get_syscall_ret(&self) -> usize {
        self.general.v0
    }

    /// Set return value of syscall
    pub fn set_syscall_ret(&mut self, ret: usize) {
        self.general.v0 = ret;
    }

    /// Get syscall args
    pub fn get_syscall_args(&self) -> [usize; 6] {
        [
            self.general.a0,
            self.general.a1,
            self.general.a2,
            self.general.a3,
            0,
            0,
        ]
    }

    /// Set instruction pointer
    pub fn set_ip(&mut self, ip: usize) {
        self.epc = ip;
    }

    /// Set stack pointer
    pub fn set_sp(&mut self, sp: usize) {
        self.general.sp = sp;
    }

    /// Get stack pointer
    pub fn get_sp(&self) -> usize {
        self.general.sp
    }

    /// Set tls pointer
    pub fn set_tls(&mut self, tls: usize) {
        self.general.s0 = tls;
    }

    pub fn run(&mut self) {
        unsafe { run_user(self) }
    }
}

#[allow(improper_ctypes)]
extern "C" {
    // fn trap_entry();
    fn run_user(regs: &mut UserContext);
}
