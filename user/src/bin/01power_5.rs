#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::yield_now;

const LEN: usize = 100;

static mut S: [u64; LEN] = [0u64; LEN];

#[no_mangle]
unsafe fn main() -> i32 {
    let p = 5u64;
    let m = 998244353u64;
    let iter: usize = 210000;
    let mut cur = 0usize;
    S[cur] = 1;
    for i in 1..=iter {
        let next = if cur + 1 == LEN { 0 } else { cur + 1 };
        S[next] = S[cur] * p % m;
        cur = next;
        if i % 10000 == 0 {
            println!("power_5 [{}/{}]", i, iter);
            yield_now();
        }
    }
    println!("{}^{} = {}(MOD {})", p, iter, S[cur], m);
    println!("Test power_5 OK!");
    0
}
