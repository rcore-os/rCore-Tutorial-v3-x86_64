use core::fmt::{self, Write};
use uart_16550::SerialPort;

static mut SERIAL_PORT: SerialPort = unsafe { SerialPort::new(0x3F8) };

pub fn init() {
  unsafe { SERIAL_PORT.init(); }
}

struct Stdout;

impl Write for Stdout {
  fn write_str(&mut self, s: &str) -> fmt::Result {
    for &c in s.as_bytes() {
      unsafe { SERIAL_PORT.send(c); }
    }
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
