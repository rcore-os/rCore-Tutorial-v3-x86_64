core::arch::global_asm!(include_str!("link_app.S"));

extern "C" {
  static _app_count: usize;
}

pub const MAX_APP_NUM: usize = 16;
const APP_BASE_ADDRESS: usize = 0xffffff0001000000;
const APP_SIZE_LIMIT: usize = 0x20000;

#[no_mangle]
#[link_section = "._user_section"]
pub static APP_DST: [u8; APP_SIZE_LIMIT * MAX_APP_NUM] = [0; APP_SIZE_LIMIT * MAX_APP_NUM];

pub fn get_app_count() -> usize {
  unsafe { _app_count }
}

pub fn get_app_name(app_id: usize) -> &'static str {
  unsafe {
    let app_0_start_ptr = (&_app_count as *const usize).add(1);
    assert!(app_id < get_app_count());
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

pub fn load_app(app_id: usize) -> (usize, usize) {
  assert!(app_id < get_app_count());
  let entry = APP_BASE_ADDRESS + app_id * APP_SIZE_LIMIT;
  let app_data = get_app_data(app_id);
  let app_dst = unsafe { core::slice::from_raw_parts_mut(entry as *mut u8, app_data.len()) };
  app_dst.copy_from_slice(app_data);
  (entry, entry + APP_SIZE_LIMIT)
}

pub fn list_apps() {
  assert_eq!(APP_DST.as_ptr() as usize, APP_BASE_ADDRESS);
  println!("/**** APPS ****");
  let app_count = get_app_count();
  for i in 0..app_count {
    let data = get_app_data(i);
    println!("{}: [{:?}, {:?})", get_app_name(i), data.as_ptr_range().start, data.as_ptr_range().end);
  }
  println!("**************/");
}