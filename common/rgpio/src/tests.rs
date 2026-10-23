// Licensed under the Apache-2.0 license

use super::*;
use core::cell::RefCell;
use janus::JANUS_PIN_MAP;
use tock_registers::interfaces::{Readable, Writeable};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Op {
    Psor(u32),
    Pcor(u32),
    Pddr(u32),
    Pidr(u32),
}

/// In-memory RGPIO model that records writes in order.
#[derive(Default)]
struct MockPort {
    pdor: RefCell<u32>,
    pddr: RefCell<u32>,
    pidr: RefCell<u32>,
    log: RefCell<Vec<Op>>,
}

impl RgpioPort for MockPort {
    fn pdor(&self) -> u32 {
        *self.pdor.borrow()
    }
    fn pddr(&self) -> u32 {
        *self.pddr.borrow()
    }
    fn pidr(&self) -> u32 {
        *self.pidr.borrow()
    }
    fn set_psor(&self, v: u32) {
        *self.pdor.borrow_mut() |= v;
        self.log.borrow_mut().push(Op::Psor(v));
    }
    fn set_pcor(&self, v: u32) {
        *self.pdor.borrow_mut() &= !v;
        self.log.borrow_mut().push(Op::Pcor(v));
    }
    fn set_pddr(&self, v: u32) {
        *self.pddr.borrow_mut() = v;
        self.log.borrow_mut().push(Op::Pddr(v));
    }
    fn set_pidr(&self, v: u32) {
        *self.pidr.borrow_mut() = v;
        self.log.borrow_mut().push(Op::Pidr(v));
    }
}

#[derive(Default)]
struct RecMux(RefCell<Vec<(GpioPort, u8, AltFn)>>);
impl PinMux for RecMux {
    fn set_alt(&self, port: GpioPort, pin: u8, alt: AltFn) -> Result<(), GpioError> {
        self.0.borrow_mut().push((port, pin, alt));
        Ok(())
    }
}

#[test]
fn output_value_set_before_direction() {
    let p = MockPort {
        pdor: RefCell::new(0b10),
        pidr: RefCell::new(0xffff_ffff),
        ..Default::default()
    };
    let map = [
        PinConfig::new(GpioPort::Gpio0, 0, PinFunction::output(Level::High)),
        PinConfig::new(GpioPort::Gpio0, 1, PinFunction::output(Level::Low)),
        PinConfig::new(GpioPort::Gpio0, 2, PinFunction::input()),
        PinConfig::new(GpioPort::Gpio0, 3, PinFunction::input_disabled()),
    ];
    gpio_init(&map, |_| Some(&p), &RecMux::default()).unwrap();
    assert_eq!(
        *p.log.borrow(),
        vec![
            Op::Psor(1 << 0),
            Op::Pddr(0b01),
            Op::Pcor(1 << 1),
            Op::Pddr(0b11),
            Op::Pddr(0b11),
            Op::Pidr(!(1 << 2)),
            Op::Pddr(0b11),
            Op::Pidr(!(1 << 2)),
        ]
    );
    assert_eq!(p.pdor(), 0b01);
    assert_eq!(p.pddr(), 0b11);
}

#[test]
fn writes_regardless_of_lock_and_missing_port_errors() {
    let p = MockPort::default();
    let map = [PinConfig::new(
        GpioPort::Gpio1,
        7,
        PinFunction::output(Level::High),
    )];
    gpio_init(&map, |_| Some(&p), &RecMux::default()).unwrap();
    assert_eq!(*p.log.borrow(), vec![Op::Psor(1 << 7), Op::Pddr(1 << 7)]);
    assert_eq!(
        gpio_init::<MockPort, _>(&map, |_| None, &RecMux::default()),
        Err(GpioError::NoSuchPort(GpioPort::Gpio1))
    );
}

#[test]
fn default_map_is_all_inputs() {
    assert_eq!(JANUS_PIN_MAP.len(), 7 * 32);
    for (i, e) in JANUS_PIN_MAP.iter().enumerate() {
        assert_eq!(e.port, GpioPort::ALL[i / 32]);
        assert_eq!(e.pin as usize, i % 32);
        assert_eq!(e.function, PinFunction::input());
    }
    let p = MockPort {
        pddr: RefCell::new(0xffff_ffff),
        pidr: RefCell::new(0xffff_ffff),
        ..Default::default()
    };
    let mux = RecMux::default();
    gpio_init(JANUS_PIN_MAP, |_| Some(&p), &mux).unwrap();
    assert!(mux.0.borrow().is_empty());
    assert_eq!(p.pddr(), 0);
    assert_eq!(p.pidr(), 0);
    assert!(!p
        .log
        .borrow()
        .iter()
        .any(|op| matches!(op, Op::Psor(_) | Op::Pcor(_))));
}

#[test]
fn set_pin_overrides_one_entry() {
    use janus::{all_inputs, set_pin, JanusPinMap};
    const MAP: JanusPinMap = set_pin(
        all_inputs(),
        GpioPort::Gpio1,
        4,
        PinFunction::Alt(AltFn::Alt2),
    );
    const MAP2: JanusPinMap = set_pin(MAP, GpioPort::Gpio6, 31, PinFunction::output(Level::High));
    assert_eq!(
        MAP2[32 + 4],
        PinConfig::new(GpioPort::Gpio1, 4, PinFunction::Alt(AltFn::Alt2))
    );
    assert_eq!(MAP2[6 * 32 + 31].function, PinFunction::output(Level::High));
    assert_eq!(
        MAP2.iter()
            .filter(|e| e.function != PinFunction::input())
            .count(),
        2
    );
}

#[test]
fn stub_mux_reports_alt_unavailable() {
    let p = MockPort::default();
    let map = [PinConfig::new(
        GpioPort::Gpio2,
        9,
        PinFunction::Alt(AltFn::Alt5),
    )];
    assert_eq!(
        gpio_init(&map, |_| Some(&p), &NoPinMux),
        Err(GpioError::PinMuxUnavailable(GpioPort::Gpio2, 9))
    );
}

#[test]
fn real_register_block_in_memory() {
    // tock-registers block backed by ordinary memory.
    let regs: Box<regs::RgpioRegisters> = unsafe { Box::new(core::mem::zeroed()) };
    let map = [PinConfig::new(
        GpioPort::Gpio4,
        5,
        PinFunction::output(Level::Low),
    )];
    gpio_init(&map, |_| Some(&*regs), &RecMux::default()).unwrap();
    assert_eq!(regs.pddr.get(), 1 << 5);
    assert_eq!(regs.pcor.get(), 1 << 5);
}

fn zeroed_port() -> Box<port::PortRegisters> {
    unsafe { Box::new(core::mem::zeroed()) }
}

#[test]
fn port_mux_writes_pcr_mux_field() {
    use port::{PortPinMux, PCR_MUX_MASK};
    let p0 = zeroed_port();
    let p1 = zeroed_port();
    // Pre-existing non-MUX bits must be preserved, old MUX bits replaced.
    p1.pcr[4].set(0xF00 | 0x3);
    let mux = PortPinMux::new(|p| match p {
        GpioPort::Gpio0 => Some(&*p0),
        GpioPort::Gpio1 => Some(&*p1),
        _ => None,
    });
    let gpio = MockPort::default();
    let map = [
        PinConfig::new(GpioPort::Gpio1, 4, PinFunction::Alt(AltFn::Alt2)),
        PinConfig::new(GpioPort::Gpio1, 31, PinFunction::Alt(AltFn::Alt15)),
        PinConfig::new(GpioPort::Gpio0, 2, PinFunction::input()),
    ];
    gpio_init(&map, |_| Some(&gpio), &mux).unwrap();
    assert_eq!(p1.pcr[4].get(), (2 << 8) | 0x3);
    assert_eq!(p1.pcr[31].get(), 0xF << 8);
    assert_eq!(
        p0.pcr[2].get() & PCR_MUX_MASK,
        (port::GPIO_MUX_ALT as u32) << 8
    );
    assert!((0..32)
        .filter(|&i| i != 4 && i != 31)
        .all(|i| p1.pcr[i].get() == 0));
    // Unmapped port -> error.
    let bad = [PinConfig::new(
        GpioPort::Gpio5,
        0,
        PinFunction::Alt(AltFn::Alt1),
    )];
    assert_eq!(
        gpio_init(&bad, |_| Some(&gpio), &mux),
        Err(GpioError::PinMuxUnavailable(GpioPort::Gpio5, 0))
    );
}

#[test]
fn janus_addresses() {
    use janus::*;
    assert_eq!(janus_gpio_base(GpioPort::Gpio5), 0x4082_0000);
    assert_eq!(janus_gpio_base(GpioPort::Gpio6), 0x4040_E000);
    assert_eq!(janus_port_base(GpioPort::Gpio0), 0x5000_0000);
    assert_eq!(janus_port_base(GpioPort::Gpio5), 0x5000_5000);
}
