// Licensed under the Apache-2.0 license

use super::*;
use core::cell::RefCell;
use janus::JANUS_PIN_MAP;

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
    lock: u32,
    pdor: RefCell<u32>,
    pddr: RefCell<u32>,
    pidr: RefCell<u32>,
    log: RefCell<Vec<Op>>,
}

impl RgpioPort for MockPort {
    fn lock(&self) -> u32 {
        self.lock
    }
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
    gpio_init(&map, |_| Some(&p), &NoPinMux).unwrap();
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
fn locked_port_is_untouched() {
    let p = MockPort {
        lock: 1,
        ..Default::default()
    };
    let map = [PinConfig::new(
        GpioPort::Gpio1,
        7,
        PinFunction::output(Level::High),
    )];
    assert_eq!(
        gpio_init(&map, |_| Some(&p), &NoPinMux),
        Err(GpioError::PortLocked(GpioPort::Gpio1))
    );
    assert!(p.log.borrow().is_empty());
}

#[test]
fn alt_functions_go_to_mux() {
    let p = MockPort::default();
    let mux = RecMux::default();
    gpio_init(JANUS_PIN_MAP, |_| Some(&p), &mux).unwrap();
    assert_eq!(
        *mux.0.borrow(),
        vec![
            (GpioPort::Gpio1, 4, AltFn::Alt2),
            (GpioPort::Gpio1, 5, AltFn::Alt2)
        ]
    );
    // pin 31 input with buffer disabled
    assert_eq!(p.pidr() & (1 << 31), 1 << 31);
    assert_eq!(p.pddr(), 0b11);
    assert_eq!(p.pdor(), 0b10);
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
    gpio_init(&map, |_| Some(&*regs), &NoPinMux).unwrap();
    assert_eq!(regs.pddr.get(), 1 << 5);
    assert_eq!(regs.pcor.get(), 1 << 5);
}
