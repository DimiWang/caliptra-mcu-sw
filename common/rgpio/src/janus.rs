// Licensed under the Apache-2.0 license

//! Janus SoC RGPIO instances and the example pin map.
//!
//! Edit [`JANUS_PIN_MAP`] to change the board pin configuration.

#[allow(unused_imports)] // AltFn/Level are used by commented-out overrides
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
/// GPIO5, non-secure alias.
pub const GPIO5_BASE: usize = 0x4082_0000;
/// GPIO6 (second AHB peripheral bus), non-secure alias.
pub const GPIO6_BASE: usize = 0x4040_E000;

/// All Janus GPIO ports, indexed by [`GpioPort`].
pub const JANUS_GPIO_BASES: [(GpioPort, usize); 7] = [
    (GpioPort::Gpio0, GPIO0_BASE),
    (GpioPort::Gpio1, GPIO1_BASE),
    (GpioPort::Gpio2, GPIO2_BASE),
    (GpioPort::Gpio3, GPIO3_BASE),
    (GpioPort::Gpio4, GPIO4_BASE),
    (GpioPort::Gpio5, GPIO5_BASE),
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
        GpioPort::Gpio5 => GPIO5_BASE,
        GpioPort::Gpio6 => GPIO6_BASE,
    }
}

/// PORT/IOMUX block base for GPIO0's port.
///
/// TODO(placeholder): the real Janus PORT/IOMUX address is unknown. Assumes one
/// PORT block per GPIO port at `JANUS_PORT_BASE + port_index * JANUS_PORT_STRIDE`.
pub const JANUS_PORT_BASE: usize = 0x5000_0000;
/// TODO(placeholder): spacing between per-port PORT blocks.
pub const JANUS_PORT_STRIDE: usize = 0x1000;

/// PORT/IOMUX block base for a GPIO port (placeholder addressing).
pub const fn janus_port_base(port: GpioPort) -> usize {
    JANUS_PORT_BASE + port.index() * JANUS_PORT_STRIDE
}

/// Number of Janus GPIO ports (GPIO0..GPIO6).
pub const JANUS_PORT_COUNT: usize = 7;
/// Entries in the full pin map (every pin of every port).
pub const JANUS_PIN_COUNT: usize = JANUS_PORT_COUNT * 32;

/// Full pin map: every pin 0..31 of GPIO0..GPIO6, one entry each.
pub type JanusPinMap = [PinConfig; JANUS_PIN_COUNT];

/// All pins as GPIO inputs (input buffer enabled).
pub const fn all_inputs() -> JanusPinMap {
    let mut map = [PinConfig::new(GpioPort::Gpio0, 0, PinFunction::input()); JANUS_PIN_COUNT];
    let mut i = 0;
    while i < JANUS_PIN_COUNT {
        map[i] = PinConfig::new(GpioPort::ALL[i / 32], (i % 32) as u8, PinFunction::input());
        i += 1;
    }
    map
}

/// Replace the function of one pin in a full map (usable in `const`).
pub const fn set_pin(
    mut map: JanusPinMap,
    port: GpioPort,
    pin: u8,
    function: PinFunction,
) -> JanusPinMap {
    map[port.index() * 32 + pin as usize] = PinConfig::new(port, pin, function);
    map
}

/// Janus board pin map. Every pin defaults to an input (input enabled).
///
/// To change a pin, wrap the default in `set_pin`, for example:
/// ```ignore
/// const MAP: JanusPinMap = set_pin(all_inputs(), GpioPort::Gpio1, 4, PinFunction::Alt(AltFn::Alt2));
/// const MAP: JanusPinMap = set_pin(MAP, GpioPort::Gpio0, 0, PinFunction::output(Level::Low));
/// ```
const JANUS_PIN_MAP_ARRAY: JanusPinMap = {
    let map = all_inputs();
    // Board-specific overrides go here, one line per pin, e.g.:
    // let map = set_pin(map, GpioPort::Gpio0, 0, PinFunction::output(Level::Low));
    // let map = set_pin(map, GpioPort::Gpio1, 4, PinFunction::Alt(AltFn::Alt2));
    map
};

/// The pin map used by `janus_gpio_init` in the ROMs.
pub const JANUS_PIN_MAP: &[PinConfig] = &JANUS_PIN_MAP_ARRAY;
