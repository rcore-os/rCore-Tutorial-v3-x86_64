use xmas_elf::{program::{SegmentData, Type}, {header, ElfFile}};

use crate::mm::*;

core::arch::global_asm!(include_str!("link_app.S"));

pub const USTACK_SIZE: usize = 4096 * 4;
pub const USTACK_TOP: usize = 0x8000_0000_0000;

extern "C" {
  static _app_count: usize;
}

pub fn get_app_count() -> usize {
  unsafe { _app_count }
}

pub fn get_app_name(app_id: usize) -> &'static str {
  assert!(app_id < get_app_count());
  unsafe {
    let app_0_start_ptr = (&_app_count as *const usize).add(1);
    let name = *app_0_start_ptr.add(app_id * 2) as *const u8;
    let mut len = 0;
    while *name.add(len) != b'\0' {
      len += 1;
    }
    let slice = core::slice::from_raw_parts(name, len);
    core::str::from_utf8_unchecked(slice)
  }
}

pub fn get_app_data(app_id: usize) -> &'static [u8] {
  assert!(app_id < get_app_count());
  unsafe {
    let app_0_start_ptr = (&_app_count as *const usize).add(1);
    let app_start = *app_0_start_ptr.add(app_id * 2 + 1);
    let app_end = *app_0_start_ptr.add(app_id * 2 + 2);
    let app_size = app_end - app_start;
    core::slice::from_raw_parts(app_start as _, app_size)
  }
}

pub fn get_app_data_by_name(name: &str) -> Option<&'static [u8]> {
  (0..get_app_count()).find(|&i| get_app_name(i) == name).map(get_app_data)
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

pub fn list_apps() {
  println!("/**** APPS ****");
  let app_count = get_app_count();
  for i in 0..app_count {
    let data = get_app_data(i);
    println!("{}: [{:?}, {:?})", get_app_name(i), data.as_ptr_range().start, data.as_ptr_range().end);
  }
  println!("**************/");
}
