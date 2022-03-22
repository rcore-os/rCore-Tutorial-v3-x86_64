use crate::x86_64::*;
use core::fmt::{self, Write};

bitflags::bitflags! {
  struct LineSts: u8 {
    const INPUT_FULL = 1;
    const OUTPUT_EMPTY = 1 << 5;
  }
}

/// A port-mapped UART. Copied from uart_16550.
const SERIAL_DATA: u16 = 0x3F8;
const SERIAL_INT_EN: u16 = SERIAL_DATA + 1;
const SERIAL_FIFO_CTRL: u16 = SERIAL_DATA + 2;
const SERIAL_LINE_CTRL: u16 = SERIAL_DATA + 3;
const SERIAL_MODEM_CTRL: u16 = SERIAL_DATA + 4;
const SERIAL_LINE_STS: u16 = SERIAL_DATA + 5;

/// Initializes the serial port.
pub fn init() {
  // Disable interrupts
  out8(SERIAL_INT_EN, 0x00);
  // Enable DLAB
  out8(SERIAL_LINE_CTRL, 0x80);
  // Set maximum speed to 38400 bps by configuring DLL and DLM
  out8(SERIAL_DATA, 0x03);
  out8(SERIAL_INT_EN, 0x00);
  // Disable DLAB and set data word length to 8 bits
  out8(SERIAL_LINE_CTRL, 0x03);
  // Enable FIFO, clear TX/RX queues and
  // set interrupt watermark at 14 bytes
  out8(SERIAL_FIFO_CTRL, 0xC7);
  // Mark data terminal ready, signal request to send
  // and enable auxilliary output #2 (used as interrupt line for CPU)
  out8(SERIAL_MODEM_CTRL, 0x0B);
  // Enable interrupts
  out8(SERIAL_INT_EN, 0x01);
}

fn line_sts() -> LineSts {
  LineSts::from_bits_truncate(in8(SERIAL_LINE_STS))
}

/// Sends a byte on the serial port.
pub fn send(data: u8) {
  match data {
    8 | 0x7F => {
      while !line_sts().contains(LineSts::OUTPUT_EMPTY) {}
      out8(SERIAL_DATA, 8);
      while !line_sts().contains(LineSts::OUTPUT_EMPTY) {}
      out8(SERIAL_DATA, b' ');
      while !line_sts().contains(LineSts::OUTPUT_EMPTY) {}
      out8(SERIAL_DATA, 8)
    }
    _ => {
      while !line_sts().contains(LineSts::OUTPUT_EMPTY) {}
      out8(SERIAL_DATA, data);
    }
  }
}

/// Receives a byte on the serial port.
pub fn receive() -> Option<u8> {
  if line_sts().contains(LineSts::INPUT_FULL) { Some(in8(SERIAL_DATA)) } else {None  }
}

struct Stdout;

impl Write for Stdout {
  fn write_str(&mut self, s: &str) -> fmt::Result {
    for &c in s.as_bytes() { send(c); }
    Ok(())
  }
}

pub fn print(args: fmt::Arguments) {
  Stdout.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
  ($fmt: literal $(, $($arg: tt)+)?) => {
    $crate::console::print(format_args!($fmt $(, $($arg)+)?))
  }
}

#[macro_export]
macro_rules! println {
  ($fmt: literal $(, $($arg: tt)+)?) => {
    $crate::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?))
  }
}
