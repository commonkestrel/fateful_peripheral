#![doc = include_str!("../README.md")]

pub use fateful_macros::*;

use std::{ cell::RefCell, thread_local, ffi::{c_int, c_char}};
use anyhow::Error;

thread_local! {
    pub static LAST_ERROR: RefCell<Option<Error>> = RefCell::new(None);
}

#[allow(unused_variables)]
pub trait Peripheral: Sized {    
    fn init(n: u8) -> anyhow::Result<Self>;

    fn drop(self) {}

    fn read(&mut self, n: u8) -> u8 {
        0x00
    }
    fn write(&mut self, n: u8, data: u8) {}
    fn reset(&mut self) {}
}

macro_rules! bail {
    ($ret:expr, $err:expr $(, $($arg:tt)*)?) => {
        $crate::update_last_error(anyhow::anyhow!($err, $($arg)*));
        return $ret;
    };
}


pub fn update_last_error<E: Into<Error>>(err: E) {
    LAST_ERROR.with(|last| *last.borrow_mut() = Some(err.into()));
}

pub fn last_error_length() -> c_int {
    LAST_ERROR.with(|last| {
        last.borrow().as_ref().map(|err| err.to_string().len() + 1).unwrap_or(0)
    }) as c_int
}

fn get_error_message() -> Option<String> {
    LAST_ERROR.with(|last| last.borrow().as_ref().map(|err| err.to_string()))
}

pub unsafe fn get_last_error(buf: *mut c_char, length: c_int) -> c_int {
    if buf.is_null() {
        bail!(-1, "recieved a null pointer where it wasn't expected");
    }

    let buffer = std::slice::from_raw_parts_mut(buf as *mut u8, length as usize);

    let error_message: Vec<u8> = match get_error_message() {
        Some(err) => err.into(),
        None => return 0,
    };

    if error_message.len() + 1 > buffer.len() {
        bail!(-1, "buffer not long enough for error message");
    }

    buffer[..error_message.len()].copy_from_slice(&error_message);
    buffer[error_message.len()] = b'\0';

    (error_message.len() + 1) as c_int
}
