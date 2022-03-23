use crate::{*, mm::PhysFrame};
use isomorphic_drivers::{provider, block::ahci::{AHCI, BLOCK_SIZE}};

pub struct AHCIDriver(Cell<AHCI<Provider>>);

impl AHCIDriver {
  pub fn new(header: usize, size: usize) -> Option<Self> {
    AHCI::new(header, size).map(|x| Self(Cell::new(x)))
  }
}

impl BlockDevice for AHCIDriver {
  fn read_block(&self, block_id: usize, buf: &mut [u8]) {
    self.0.get().read_block(block_id, buf);
  }

  fn write_block(&self, block_id: usize, buf: &[u8]) {
    assert!(buf.len() >= BLOCK_SIZE);
    self.0.get().write_block(block_id, buf);
  }
}

struct Provider;

impl provider::Provider for Provider {
  const PAGE_SIZE: usize = mm::PAGE_SIZE;

  fn alloc_dma(size: usize) -> (usize, usize) {
    println!("alloc_dma: {:x}", size);
    let pages = size / mm::PAGE_SIZE;
    let mut base = 0;
    for i in 0..pages {
      let frame = PhysFrame::alloc().unwrap();
      let frame_pa = frame.start_pa().0;
      core::mem::forget(frame);
      if i == 0 { base = frame_pa; }
      assert_eq!(frame_pa, base + i * mm::PAGE_SIZE);
    }
    println!("virtio_dma_alloc: {:x} {}", base, pages);
    (mm::phys_to_virt(base), base)
  }

  fn dealloc_dma(va: usize, size: usize) {
    println!("dealloc_dma: {:x} {:x}", va, size);
    let pages = size / mm::PAGE_SIZE;
    let mut pa = mm::virt_to_phys(va);
    for _ in 0..pages {
      PhysFrame::dealloc(pa);
      pa += mm::PAGE_SIZE;
    }
  }
}
