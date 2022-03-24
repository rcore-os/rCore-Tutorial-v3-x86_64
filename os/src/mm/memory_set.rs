use crate::*;
use super::*;
use core::fmt;
use alloc::collections::btree_map::Entry;
use xmas_elf::{program::{SegmentData, Type}, {header, ElfFile}};

pub const USTACK_SIZE: usize = 4096 * 4;
pub const USTACK_TOP: usize = 0x8000_0000_0000;

pub struct MapArea {
  pub start: VirtAddr,
  pub size: usize,
  pub flags: PTFlags,
  pub mapper: BTreeMap<VirtAddr, PhysFrame>,
}

pub struct MemorySet {
  pub pt: PageTable,
  areas: BTreeMap<VirtAddr, MapArea>,
}

impl MapArea {
  pub fn new(start_va: VirtAddr, size: usize, flags: PTFlags) -> Self {
    assert!(start_va.is_aligned() && is_aligned(size));
    Self { start: start_va, size, flags, mapper: BTreeMap::new() }
  }

  pub fn clone(&self) -> Self {
    let mut mapper = BTreeMap::new();
    for (&va, old) in &self.mapper {
      let new = PhysFrame::alloc().unwrap();
      new.as_slice().copy_from_slice(old.as_slice());
      mapper.insert(va, new);
    }
    Self { start: self.start, size: self.size, flags: self.flags, mapper }
  }

  pub fn map(&mut self, va: VirtAddr) -> PhysAddr {
    assert!(va.is_aligned());
    match self.mapper.entry(va) {
      Entry::Occupied(e) => e.get().start_pa(),
      Entry::Vacant(e) => e.insert(PhysFrame::alloc_zero().unwrap()).start_pa(),
    }
  }

  pub fn unmap(&mut self, va: VirtAddr) {
    self.mapper.remove(&va);
  }

  pub fn write_data(&mut self, offset: usize, data: &[u8]) {
    assert!(offset + data.len() < self.size);
    let mut start = offset;
    let mut remain = data.len();
    let mut processed = 0;
    while remain > 0 {
      let start_align = align_down(start);
      let pgoff = start - start_align;
      let n = (PAGE_SIZE - pgoff).min(remain);
      let pa = self.map(VirtAddr(self.start.0 + start_align));
      unsafe {
        core::slice::from_raw_parts_mut(pa.kvaddr().as_ptr().add(pgoff), n)
          .copy_from_slice(&data[processed..processed + n]);
      }
      start += n;
      processed += n;
      remain -= n;
    }
  }
}

impl MemorySet {
  pub fn new() -> Self {
    Self { pt: PageTable::new(), areas: BTreeMap::new() }
  }

  pub fn insert(&mut self, area: MapArea) {
    if area.size > 0 {
      // TODO: check overlap
      if let Entry::Vacant(e) = self.areas.entry(area.start) {
        self.pt.map_area(e.insert(area));
      } else {
        panic!("MemorySet::insert: MapArea starts from {:#x?} is existed!", area.start);
      }
    }
  }

  pub fn clear(&mut self) {
    for area in self.areas.values_mut() {
      self.pt.unmap_area(area);
    }
    self.areas.clear();
  }

  pub fn activate(&self) {
    x86_64::set_cr3(self.pt.root_pa.0);
  }
}

impl Clone for MemorySet {
  fn clone(&self) -> Self {
    let mut ms = Self::new();
    for area in self.areas.values() { ms.insert(area.clone()); }
    ms
  }
}

impl Drop for MemorySet {
  fn drop(&mut self) {
    self.clear();
  }
}

impl fmt::Debug for MapArea {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.debug_struct("MapArea")
      .field("va_range", &(self.start.0..self.start.0 + self.size))
      .field("flags", &self.flags)
      .finish()
  }
}

impl fmt::Debug for MemorySet {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.debug_struct("MemorySet")
      .field("areas", &self.areas.values())
      .field("page_table_root", &self.pt.root_pa)
      .finish()
  }
}

pub fn load_app(elf_data: &[u8]) -> (usize, MemorySet) {
  let elf = ElfFile::new(elf_data).expect("invalid ELF file");
  assert_eq!(elf.header.pt1.class(), header::Class::SixtyFour, "64-bit ELF required");
  assert_eq!(elf.header.pt2.type_().as_type(), header::Type::Executable, "ELF is not an executable object");
  assert_eq!(elf.header.pt2.machine().as_machine(), header::Machine::X86_64, "invalid ELF arch");
  let mut ms = MemorySet::new();
  for ph in elf.program_iter() {
    if ph.get_type() != Ok(Type::Load) {
      continue;
    }
    let va = VirtAddr(ph.virtual_addr() as _);
    let offset = va.page_offset();
    let area_start = va.align_down();
    let area_end = VirtAddr((ph.virtual_addr() + ph.mem_size()) as _).align_up();
    let data = match ph.get_data(&elf).unwrap() {
      SegmentData::Undefined(data) => data,
      _ => panic!("failed to get ELF segment data"),
    };

    let mut flags = PTFlags::PRESENT | PTFlags::USER;
    if ph.flags().is_write() {
      flags |= PTFlags::WRITABLE;
    }
    let mut area = MapArea::new(area_start, area_end.0 - area_start.0, flags);
    area.write_data(offset, data);
    ms.insert(area);
  }
  ms.insert(MapArea::new(VirtAddr(USTACK_TOP - USTACK_SIZE), USTACK_SIZE,
    PTFlags::PRESENT | PTFlags::WRITABLE | PTFlags::USER));
  (elf.header.pt2.entry_point() as usize, ms)
}
