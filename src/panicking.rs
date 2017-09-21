// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Panic support for libcore
//!
//! The core library cannot define panicking, but it does *declare* panicking. This
//! means that the functions inside of libcore are allowed to panic, but to be
//! useful an upstream crate must define panicking for libcore to use. The current
//! interface for panicking is:
//!
//! ```ignore
//! fn panic_impl(fmt: fmt::Arguments, &(&'static str, u32)) -> !;
//! ```
//!
//! This definition allows for panicking with any general message, but it does not
//! allow for failing with a `Box<Any>` value. The reason for this is that libcore
//! is not allowed to allocate.
//!
//! This module contains a few other panicking functions, but these are just the
//! necessary lang items for the compiler. All panics are funneled through this
//! one function. Currently, the actual symbol is declared in the standard
//! library, but the location of this may change over time.

#![allow(dead_code, missing_docs)]
#![unstable(feature = "core_panic",
            reason = "internal details of the implementation of the `panic!` \
                      and related macros",
            issue = "0")]

use fmt;

#[cold] #[inline(never)] // this is the slow path, always
#[lang = "panic"]
pub fn panic(expr_file_line: &(&'static str, &'static str, u32)) -> ! {
    use ptr::{read_volatile, write_volatile};
    loop {
        unsafe {
            #[allow(non_snake_case)]
            let DDRC : *mut u8 = 0x27 as *mut u8;
            #[allow(non_snake_case)]
            let PORTC : *mut u8 = 0x28 as *mut u8;
            #[cfg(debug_assertions)]
            const DELAY : u32 = 20000;
            #[cfg(not(debug_assertions))]
            const DELAY : u32 = 700000;
            write_volatile(DDRC, read_volatile(DDRC) | 0b11);
            for _ in 0..DELAY {
                write_volatile(PORTC, read_volatile(PORTC) & 0xfc);
            }
            for _ in 0..DELAY {
                write_volatile(PORTC, read_volatile(PORTC) | 0b11);
            }
        }
    }
}

#[cold] #[inline(never)]
#[lang = "panic_bounds_check"]
fn panic_bounds_check(file_line: &(&'static str, u32),
                     index: usize, len: usize) -> ! {
    panic(&("...", "...", 0));
}

#[cold] #[inline(never)]
pub fn panic_fmt(fmt: fmt::Arguments, file_line: &(&'static str, u32)) -> ! {
    #[allow(improper_ctypes)]
    extern {
        #[lang = "panic_fmt"]
        #[unwind]
        fn panic_impl(fmt: fmt::Arguments, file: &'static str, line: u32) -> !;
    }
    let (file, line) = *file_line;
    unsafe { panic_impl(fmt, file, line) }
}
