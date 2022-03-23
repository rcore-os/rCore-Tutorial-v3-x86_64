use crate::*;
use super::ahci::AHCIDriver;
use pci::*;

const PCI_COMMAND: u16 = 0x04;
const PCI_CAP_PTR: u16 = 0x34;
const PCI_INTERRUPT_LINE: u16 = 0x3c;
const PCI_INTERRUPT_PIN: u16 = 0x3d;

const PCI_MSI_CTRL_CAP: u16 = 0x00;
const PCI_MSI_ADDR: u16 = 0x04;
const PCI_MSI_UPPER_ADDR: u16 = 0x08;
const PCI_MSI_DATA_32: u16 = 0x08;
const PCI_MSI_DATA_64: u16 = 0x0C;

const PCI_CAP_ID_MSI: u8 = 0x05;

struct PortOpsImpl;

impl PortOps for PortOpsImpl {
  unsafe fn read8(&self, port: u16) -> u8 { x86_64::in8(port) }
  unsafe fn read16(&self, port: u16) -> u16 { x86_64::in16(port) }
  unsafe fn read32(&self, port: u16) -> u32 { x86_64::in32(port) }
  unsafe fn write8(&self, port: u16, val: u8) { x86_64::out8(port, val); }
  unsafe fn write16(&self, port: u16, val: u16) { x86_64::out16(port, val); }
  unsafe fn write32(&self, port: u16, val: u32) { x86_64::out32(port, val); }
}

/// Enable the pci device and its interrupt
/// Return assigned MSI interrupt number when applicable
unsafe fn enable(loc: Location) {
  let ops = &PortOpsImpl;
  let am = CSpaceAccessMethod::IO;

  // 23 and lower are used
  static mut MSI_IRQ: u32 = 23;

  let orig = am.read16(ops, loc, PCI_COMMAND);
  // IO Space | MEM Space | Bus Mastering | Special Cycles | PCI Interrupt Disable
  am.write32(ops, loc, PCI_COMMAND, (orig | 0x40f) as u32);

  // find MSI cap
  let mut msi_found = false;
  let mut cap_ptr = am.read8(ops, loc, PCI_CAP_PTR) as u16;
  while cap_ptr > 0 {
    let cap_id = am.read8(ops, loc, cap_ptr);
    if cap_id == PCI_CAP_ID_MSI {
      let orig_ctrl = am.read32(ops, loc, cap_ptr + PCI_MSI_CTRL_CAP);
      // The manual Volume 3 Chapter 10.11 Message Signalled Interrupts
      // 0 is (usually) the apic id of the bsp.
      am.write32(ops, loc, cap_ptr + PCI_MSI_ADDR, 0xfee00000 | (0 << 12));
      MSI_IRQ += 1;
      let irq = MSI_IRQ;
      // we offset all our irq numbers by 32
      if (orig_ctrl >> 16) & (1 << 7) != 0 {
        // 64bit
        am.write32(ops, loc, cap_ptr + PCI_MSI_DATA_64, irq + 32);
      } else {
        // 32bit
        am.write32(ops, loc, cap_ptr + PCI_MSI_DATA_32, irq + 32);
      }

      // enable MSI interrupt, assuming 64bit for now
      am.write32(ops, loc, cap_ptr + PCI_MSI_CTRL_CAP, orig_ctrl | 0x10000);
      msi_found = true;
    }
    cap_ptr = am.read8(ops, loc, cap_ptr + 1) as u16;
  }

  if !msi_found {
    // Use PCI legacy interrupt instead
    // IO Space | MEM Space | Bus Mastering | Special Cycles
    am.write32(ops, loc, PCI_COMMAND, (orig | 0xf) as u32);
  }
}

pub fn init() -> Option<AHCIDriver> {
  for dev in unsafe { scan_bus(&PortOpsImpl, CSpaceAccessMethod::IO) } {
    println!("pci: {:02x}:{:02x}.{} {:#x} {:#x} ({} {}) irq: {}:{:?}",
      dev.loc.bus, dev.loc.device, dev.loc.function, dev.id.vendor_id, dev.id.device_id,
      dev.id.class, dev.id.subclass, dev.pic_interrupt_line, dev.interrupt_pin);
    if dev.id.class == 0x01 && dev.id.subclass == 0x06 {
      // Mass storage class, SATA subclass
      if let Some(BAR::Memory(pa, len, _, _)) = dev.bars[5] {
        println!("Found AHCI dev {:?} BAR5 {:x?}", dev, pa);
        unsafe { enable(dev.loc) };
        assert!(len as usize <= mm::PAGE_SIZE);
        if let Some(x) = AHCIDriver::new(mm::phys_to_virt(pa as _), len as _) {
          return Some(x);
        }
      }
    }
  }
  None
}
