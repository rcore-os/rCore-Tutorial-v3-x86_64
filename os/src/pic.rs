use crate::{*, x86_64::*};

const MASTER_CMD: u16 = 0x20;
const MASTER_DATA: u16 = MASTER_CMD + 1;
const SLAVE_CMD: u16 = 0xA0;
const SLAVE_DATA: u16 = SLAVE_CMD + 1;

const TIMER_RATE: u32 = 1193182;
const TIMER_PERIOD_IO_PORT: u16 = 0x40;
const TIMER_MODE_IO_PORT: u16 = 0x43;
const TIMER_SQUARE_WAVE: u8 = 0x36;

pub static TICKS: Cell<usize> = zero();

pub fn init() {
  // Start initialization
  out8(MASTER_CMD, 0x11);
  out8(SLAVE_CMD, 0x11);

  // Set offsets
  out8(MASTER_DATA, 0x20);
  out8(SLAVE_DATA, 0x28);

  // Set up cascade
  out8(MASTER_DATA, 4);
  out8(SLAVE_DATA, 2);

  // Set up interrupt mode (1 is 8086/88 mode, 2 is auto EOI)
  out8(MASTER_DATA, 1);
  out8(SLAVE_DATA, 1);

  // Unmask interrupts
  out8(MASTER_DATA, 0);
  out8(SLAVE_DATA, 0);

  // Ack remaining interrupts
  out8(MASTER_CMD, 0x20);
  out8(SLAVE_CMD, 0x20);

  // Initialize timer.
  let cycle = TIMER_RATE / 1000; // 1ms per interrupt.
  out8(TIMER_MODE_IO_PORT, TIMER_SQUARE_WAVE);
  out8(TIMER_PERIOD_IO_PORT, (cycle & 0xFF) as _);
  out8(TIMER_PERIOD_IO_PORT, (cycle >> 8) as _);
}

#[inline(always)]
pub fn ack() {
  out8(MASTER_CMD, 0x20);
}
