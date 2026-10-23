// Licensed under the Apache-2.0 license

//! NXP PORT-style pin mux.
//!
//! TODO(placeholder): the Janus PORT/IOMUX layout is not documented yet. This
//! assumes the NXP PORT layout: `PCR[32]` (u32, step 4) at offset 0x80 with
//! the MUX field in bits [11:8]. Swap the [`PinMux`] implementation if the
//! real block differs.

use crate::{AltFn, GpioError, GpioPort, PinMux};
use tock_registers::interfaces::{Readable, Writeable};
use tock_registers::register_structs;
use tock_registers::registers::ReadWrite;

register_structs! {
    /// Placeholder PORT block (one per GPIO port).
    pub PortRegisters {
        (0x000 => _reserved0),
        /// Pin Control Register 0..31
        (0x080 => pub pcr: [ReadWrite<u32>; 32]),
        (0x100 => @END),
    }
}

const _: () = assert!(core::mem::offset_of!(PortRegisters, pcr) == 0x80);

/// PCR.MUX field position (placeholder).
pub const PCR_MUX_SHIFT: u32 = 8;
/// PCR.MUX field mask (bits [11:8], placeholder).
pub const PCR_MUX_MASK: u32 = 0xF << PCR_MUX_SHIFT;
/// Mux value that routes a pin to RGPIO (placeholder: Alt0).
pub const GPIO_MUX_ALT: AltFn = AltFn::Alt0;

/// [`PinMux`] that writes PCR.MUX in a per-port PORT block.
pub struct PortPinMux<F> {
    lookup: F,
}

impl<'a, F: Fn(GpioPort) -> Option<&'a PortRegisters>> PortPinMux<F> {
    pub const fn new(lookup: F) -> Self {
        Self { lookup }
    }

    fn write_mux(&self, port: GpioPort, pin: u8, alt: AltFn) -> Result<(), GpioError> {
        let regs = (self.lookup)(port).ok_or(GpioError::PinMuxUnavailable(port, pin))?;
        let pcr = &regs.pcr[pin as usize];
        let v = (pcr.get() & !PCR_MUX_MASK) | ((alt as u32) << PCR_MUX_SHIFT);
        pcr.set(v);
        Ok(())
    }
}

impl<'a, F: Fn(GpioPort) -> Option<&'a PortRegisters>> PinMux for PortPinMux<F> {
    fn set_alt(&self, port: GpioPort, pin: u8, alt: AltFn) -> Result<(), GpioError> {
        self.write_mux(port, pin, alt)
    }
    fn set_gpio(&self, port: GpioPort, pin: u8) -> Result<(), GpioError> {
        self.write_mux(port, pin, GPIO_MUX_ALT)
    }
}
