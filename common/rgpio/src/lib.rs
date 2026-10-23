// Licensed under the Apache-2.0 license

//! RGPIO (NXP-style) port initialization from a static pin map.
//!
//! RGPIO has no pin-mux register; alternate functions live in a separate
//! PORT/IOMUX block. Alt-function entries are passed to a [`PinMux`]
//! implementation ([`NoPinMux`] until the PORT block is known).

#![cfg_attr(not(test), no_std)]

pub mod janus;
pub mod regs;

use regs::RgpioRegisters;
use tock_registers::interfaces::{Readable, Writeable};

/// GPIO port instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GpioPort {
    Gpio0,
    Gpio1,
    Gpio2,
    Gpio3,
    Gpio4,
    Gpio6,
}

/// Alternate function number (Alt0..Alt15).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum AltFn {
    Alt0 = 0,
    Alt1,
    Alt2,
    Alt3,
    Alt4,
    Alt5,
    Alt6,
    Alt7,
    Alt8,
    Alt9,
    Alt10,
    Alt11,
    Alt12,
    Alt13,
    Alt14,
    Alt15,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level {
    Low,
    High,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    /// Input; `input_enable = false` sets PIDR (input buffer disabled).
    Input { input_enable: bool },
    /// Output driven to `init` before the direction is switched.
    Output { init: Level },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PinFunction {
    /// Pin is muxed to an alternate (peripheral) function.
    Alt(AltFn),
    /// Pin is a GPIO.
    Gpio(Direction),
}

impl PinFunction {
    pub const fn output(init: Level) -> Self {
        Self::Gpio(Direction::Output { init })
    }
    pub const fn input() -> Self {
        Self::Gpio(Direction::Input { input_enable: true })
    }
    pub const fn input_disabled() -> Self {
        Self::Gpio(Direction::Input {
            input_enable: false,
        })
    }
}

/// One pin map entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PinConfig {
    pub port: GpioPort,
    pub pin: u8,
    pub function: PinFunction,
}

impl PinConfig {
    /// Panics at compile time (in const context) if `pin > 31`.
    pub const fn new(port: GpioPort, pin: u8, function: PinFunction) -> Self {
        assert!(pin < 32, "RGPIO pin must be 0..=31");
        Self {
            port,
            pin,
            function,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GpioError {
    /// LOCK register is non-zero; port configuration is frozen.
    PortLocked(GpioPort),
    /// The pin mux hook could not apply an alternate function.
    PinMuxUnavailable(GpioPort, u8),
}

/// Register-level access to one RGPIO port (abstracted for unit tests).
pub trait RgpioPort {
    fn lock(&self) -> u32;
    fn pdor(&self) -> u32;
    fn pddr(&self) -> u32;
    fn pidr(&self) -> u32;
    fn set_psor(&self, v: u32);
    fn set_pcor(&self, v: u32);
    fn set_pddr(&self, v: u32);
    fn set_pidr(&self, v: u32);
}

impl RgpioPort for RgpioRegisters {
    fn lock(&self) -> u32 {
        self.lock.get()
    }
    fn pdor(&self) -> u32 {
        self.pdor.get()
    }
    fn pddr(&self) -> u32 {
        self.pddr.get()
    }
    fn pidr(&self) -> u32 {
        self.pidr.get()
    }
    fn set_psor(&self, v: u32) {
        self.psor.set(v)
    }
    fn set_pcor(&self, v: u32) {
        self.pcor.set(v)
    }
    fn set_pddr(&self, v: u32) {
        self.pddr.set(v)
    }
    fn set_pidr(&self, v: u32) {
        self.pidr.set(v)
    }
}

/// Hook into the (separate) PORT/IOMUX block that selects alternate functions.
pub trait PinMux {
    fn set_alt(&self, port: GpioPort, pin: u8, alt: AltFn) -> Result<(), GpioError>;
    /// Route the pin to its GPIO function (if the mux requires it).
    fn set_gpio(&self, _port: GpioPort, _pin: u8) -> Result<(), GpioError> {
        Ok(())
    }
}

/// Placeholder until the PORT/IOMUX base address and layout are known.
/// TODO: implement once the Janus PORT block is documented.
pub struct NoPinMux;

impl PinMux for NoPinMux {
    fn set_alt(&self, port: GpioPort, pin: u8, _alt: AltFn) -> Result<(), GpioError> {
        Err(GpioError::PinMuxUnavailable(port, pin))
    }
}

/// Configure every entry of `map`.
///
/// * Ports with a non-zero LOCK are rejected before anything is written.
/// * Outputs: the init level is written via PSOR/PCOR *before* PDDR is set,
///   so the pin never glitches to a stale PDOR value.
/// * Inputs: PDDR bit cleared, PIDR bit cleared (or set if input disabled).
/// * Alt functions: forwarded to `mux`.
pub fn gpio_init<'a, P: RgpioPort + 'a, M: PinMux>(
    map: &[PinConfig],
    port_regs: impl Fn(GpioPort) -> Option<&'a P>,
    mux: &M,
) -> Result<(), GpioError> {
    // Check locks up front so a locked port leaves everything untouched.
    for e in map {
        match port_regs(e.port) {
            Some(r) if r.lock() == 0 => {}
            _ => return Err(GpioError::PortLocked(e.port)),
        }
    }
    for e in map {
        let regs = port_regs(e.port).ok_or(GpioError::PortLocked(e.port))?;
        let bit = 1u32 << e.pin;
        match e.function {
            PinFunction::Alt(alt) => mux.set_alt(e.port, e.pin, alt)?,
            PinFunction::Gpio(dir) => {
                mux.set_gpio(e.port, e.pin)?;
                match dir {
                    Direction::Output { init } => {
                        match init {
                            Level::High => regs.set_psor(bit),
                            Level::Low => regs.set_pcor(bit),
                        }
                        regs.set_pddr(regs.pddr() | bit);
                    }
                    Direction::Input { input_enable } => {
                        regs.set_pddr(regs.pddr() & !bit);
                        if input_enable {
                            regs.set_pidr(regs.pidr() & !bit);
                        } else {
                            regs.set_pidr(regs.pidr() | bit);
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

/// Initialize the Janus GPIO ports from `map` using the real MMIO blocks.
///
/// # Safety
/// Must only be called on Janus hardware where the GPIO bases are mapped.
pub unsafe fn janus_gpio_init<M: PinMux>(map: &[PinConfig], mux: &M) -> Result<(), GpioError> {
    gpio_init(
        map,
        |p| Some(&*(janus::janus_gpio_base(p) as *const RgpioRegisters)),
        mux,
    )
}

#[cfg(test)]
mod tests;
