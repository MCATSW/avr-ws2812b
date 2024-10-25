#![no_std]
#![feature(asm_experimental_arch)]

//! This crate implements a WS2812B driver
//!
//! You can get started by using the `WS2812B` struct.

use core::arch::asm;

use avr_delay::delay_us;
use avr_pin::{Pin, DD};

/// Represents a WS2812B data line.
#[repr(packed)]
#[derive(Clone, Copy)]
pub struct WS2812B {
    pub data_line_pid: u8,
}

/// Describes a `WS2812B` pixel color state.
#[repr(packed)]
#[derive(Clone, Copy)]
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RGB {
    /// Returns `Self` as bytes in protocol-defined order.
    pub fn to_bytes(self) -> [u8; 3] {
        [self.g, self.r, self.b]
    }
}

/// Sends a timed WS2812B pulse representing a 0 to a given `Pin`.
///
/// # Safety
///
/// Since the `Pin` struct can be constructed by the user,
/// there is no guarantee that the I/O register addresses
/// are valid. Please ensure validity of self, ideally by
/// avoiding manual generation of `Pin`.
pub unsafe fn send_0(pin: &Pin) {
    let high: u8 = *pin.port | pin.mask;
    let low: u8 = *pin.port & !pin.mask;
    asm!(
        // 6 TICKS TOTAL
        "st {port}, {high}", // 2 TICKS
        "nop", // 1 TICK
        "nop", // 1 TICK
        "nop", // 1 TICK
        "nop", // 1 TICK
        "st {port}, {low}", // 2 TICKS
        port = in(reg_ptr) pin.port,
        high = in(reg) high,
        low = in(reg) low,
    );
}

/// Sends a timed WS2812B pulse representing a 1 to a given `Pin`.
///
/// # Safety
///
/// Since the `Pin` struct can be constructed by the user,
/// there is no guarantee that the I/O register addresses
/// are valid. Please ensure validity of self, ideally by
/// avoiding manual generation of `Pin`.
pub unsafe fn send_1(pin: &Pin) {
    let high: u8 = *pin.port | pin.mask;
    let low: u8 = *pin.port & !pin.mask;
    asm!(
        // 13 TICKS TOTAL
        "st {port}, {high}", // 2 TICKS
        "nop", // 1 TICK
        "nop", // 1 TICK
        "nop", // 1 TICK
        "nop", // 1 TICK
        "nop", // 1 TICK
        "nop", // 1 TICK
        "nop", // 1 TICK
        "nop", // 1 TICK
        "nop", // 1 TICK
        "nop", // 1 TICK
        "nop", // 1 TICK
        "st {port}, {low}", // 2 TICKS
        port = in(reg_ptr) pin.port,
        high = in(reg) high,
        low = in(reg) low,
    );
}

impl WS2812B {
    /// Creates a new `WS2812B` instance.
    pub const fn new(data_line_pid: u8) -> Self {
        Self {
            data_line_pid,
        }
    }
    /// Initializes the WS2812B.
    ///
    /// Returns false if the pid in `Self` is invalid.
    /// Otherwise, returns true.
    pub fn init(&self) -> bool {
        let pin: Pin = match Pin::from_pid(self.data_line_pid) {
            Some(pin) => pin,
            None => return false,
        };
        unsafe {
            pin.set_ddr(DD::Output);
            pin.write(false);
        }
        true
    }
    /// Sends a pixel array to the WS2812B.
    ///
    /// Returns false if the pid in `Self` is invalid.
    /// Otherwise, returns true.
    pub fn upload(&self, buffer: &[RGB]) -> bool {
        let pin: Pin = match Pin::from_pid(self.data_line_pid) {
            Some(pin) => pin,
            None => return false,
        };
        for rgb in buffer {
            for data in (*rgb).to_bytes() {
                let mut mask: u8 = 0x80;
                while mask != 0 {
                    if data & mask > 0 {
                        unsafe { send_1(&pin); }
                    } else {
                        unsafe { send_0(&pin); }
                    }
                    delay_us(1);
                    mask >>= 1;
                }
            }
        }
        delay_us(50);
        true
    }
}

