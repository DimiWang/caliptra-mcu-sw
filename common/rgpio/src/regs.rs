// Licensed under the Apache-2.0 license

//! NXP-style RGPIO register block (`GPIO_Type`).

use core::mem::offset_of;
use tock_registers::register_structs;
use tock_registers::registers::{ReadOnly, ReadWrite};

register_structs! {
    /// One RGPIO port (32 pins). Layout matches the C `GPIO_Type` exactly.
    pub RgpioRegisters {
        /// Version ID
        (0x000 => pub verid: ReadOnly<u32>),
        /// Parameter
        (0x004 => pub param: ReadOnly<u32>),
        (0x008 => _reserved0),
        /// Lock
        (0x00C => pub lock: ReadWrite<u32>),
        /// Pin Control Nonsecure
        (0x010 => pub pcns: ReadWrite<u32>),
        /// Interrupt Control Nonsecure
        (0x014 => pub icns: ReadWrite<u32>),
        /// Pin Control Nonprivilege
        (0x018 => pub pcnp: ReadWrite<u32>),
        /// Interrupt Control Nonprivilege
        (0x01C => pub icnp: ReadWrite<u32>),
        (0x020 => _reserved1),
        /// Port Data Output
        (0x040 => pub pdor: ReadWrite<u32>),
        /// Port Set Output (write 1 to set)
        (0x044 => pub psor: ReadWrite<u32>),
        /// Port Clear Output (write 1 to clear)
        (0x048 => pub pcor: ReadWrite<u32>),
        /// Port Toggle Output (write 1 to toggle)
        (0x04C => pub ptor: ReadWrite<u32>),
        /// Port Data Input
        (0x050 => pub pdir: ReadOnly<u32>),
        /// Port Data Direction (1 = output)
        (0x054 => pub pddr: ReadWrite<u32>),
        /// Port Input Disable (1 = input disabled)
        (0x058 => pub pidr: ReadWrite<u32>),
        (0x05C => _reserved2),
        /// Pin Data
        (0x060 => pub pdr: [ReadWrite<u8>; 32]),
        /// Interrupt Control 0..31 (irregular: not all indices valid)
        (0x080 => pub icr: [ReadWrite<u32>; 32]),
        /// Global Interrupt Control Low
        (0x100 => pub giclr: ReadWrite<u32>),
        /// Global Interrupt Control High
        (0x104 => pub gichr: ReadWrite<u32>),
        (0x108 => _reserved3),
        /// Interrupt Status Flag
        (0x120 => pub isfr: [ReadWrite<u32>; 2]),
        (0x128 => @END),
    }
}

const _: () = {
    assert!(offset_of!(RgpioRegisters, verid) == 0x0);
    assert!(offset_of!(RgpioRegisters, param) == 0x4);
    assert!(offset_of!(RgpioRegisters, lock) == 0xC);
    assert!(offset_of!(RgpioRegisters, pcns) == 0x10);
    assert!(offset_of!(RgpioRegisters, icns) == 0x14);
    assert!(offset_of!(RgpioRegisters, pcnp) == 0x18);
    assert!(offset_of!(RgpioRegisters, icnp) == 0x1C);
    assert!(offset_of!(RgpioRegisters, pdor) == 0x40);
    assert!(offset_of!(RgpioRegisters, psor) == 0x44);
    assert!(offset_of!(RgpioRegisters, pcor) == 0x48);
    assert!(offset_of!(RgpioRegisters, ptor) == 0x4C);
    assert!(offset_of!(RgpioRegisters, pdir) == 0x50);
    assert!(offset_of!(RgpioRegisters, pddr) == 0x54);
    assert!(offset_of!(RgpioRegisters, pidr) == 0x58);
    assert!(offset_of!(RgpioRegisters, pdr) == 0x60);
    assert!(offset_of!(RgpioRegisters, icr) == 0x80);
    assert!(offset_of!(RgpioRegisters, giclr) == 0x100);
    assert!(offset_of!(RgpioRegisters, gichr) == 0x104);
    assert!(offset_of!(RgpioRegisters, isfr) == 0x120);
    assert!(core::mem::size_of::<RgpioRegisters>() == 0x128);
};
