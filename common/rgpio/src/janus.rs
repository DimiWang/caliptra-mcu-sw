// Licensed under the Apache-2.0 license

//! Janus SoC RGPIO instances and the example pin map.
//!
//! Edit [`JANUS_PIN_MAP`] to change the board pin configuration.

use crate::{AltFn, GpioPort, Level, PinConfig, PinFunction};

/// GPIO0 (AHB_PERIPH0), non-secure alias.
pub const GPIO0_BASE: usize = 0x4030_C000;
/// GPIO1 (second AHB peripheral bus), non-secure alias.
pub const GPIO1_BASE: usize = 0x4040_C000;
/// GPIO2 (AHB_PERIPH0), non-secure alias.
pub const GPIO2_BASE: usize = 0x4030_D000;
/// GPIO3 (second AHB peripheral bus), non-secure alias.
pub const GPIO3_BASE: usize = 0x4040_D000;
/// GPIO4 (AHB_PERIPH0), non-secure alias.
pub const GPIO4_BASE: usize = 0x4030_E000;
/// GPIO6 (second AHB peripheral bus), non-secure alias.
/// NOTE: the source table lists GPIO6 and no GPIO5; to be confirmed.
pub const GPIO6_BASE: usize = 0x4040_E000;

/// All Janus GPIO ports, indexed by [`GpioPort`].
pub const JANUS_GPIO_BASES: [(GpioPort, usize); 6] = [
    (GpioPort::Gpio0, GPIO0_BASE),
    (GpioPort::Gpio1, GPIO1_BASE),
    (GpioPort::Gpio2, GPIO2_BASE),
    (GpioPort::Gpio3, GPIO3_BASE),
    (GpioPort::Gpio4, GPIO4_BASE),
    (GpioPort::Gpio6, GPIO6_BASE),
];

/// Base address of a Janus GPIO port.
pub const fn janus_gpio_base(port: GpioPort) -> usize {
    match port {
        GpioPort::Gpio0 => GPIO0_BASE,
        GpioPort::Gpio1 => GPIO1_BASE,
        GpioPort::Gpio2 => GPIO2_BASE,
        GpioPort::Gpio3 => GPIO3_BASE,
        GpioPort::Gpio4 => GPIO4_BASE,
        GpioPort::Gpio6 => GPIO6_BASE,
    }
}

/// Example board pin map (placeholder values; edit for the real board).
#[rustfmt::skip]
pub const JANUS_PIN_MAP: &[PinConfig] = &[
    //             port             pin  function
    PinConfig::new(GpioPort::Gpio0,  0, PinFunction::output(Level::Low)),   // e.g. reset-out, held low
    PinConfig::new(GpioPort::Gpio0,  1, PinFunction::output(Level::High)),  // e.g. LED, active low
    PinConfig::new(GpioPort::Gpio0,  2, PinFunction::input()),              // e.g. button
    PinConfig::new(GpioPort::Gpio1,  4, PinFunction::Alt(AltFn::Alt2)),     // e.g. UART TX
    PinConfig::new(GpioPort::Gpio1,  5, PinFunction::Alt(AltFn::Alt2)),     // e.g. UART RX
    PinConfig::new(GpioPort::Gpio6, 31, PinFunction::input_disabled()),     // unused, input buffer off
];
