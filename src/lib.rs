#![doc = include_str!("../README.md")]

pub use fateful_macros::*;

/// A trait for implementing all available peripheral functionality.
#[allow(unused_variables)]
pub trait Peripheral: Sized { 
    /// Called when the peripheral is first loaded.
    /// 
    /// `ports` represents the number of ports this port is connected to.
    /// The `port` argument in [`read`](Peripheral::read) and [`write`](Peripheral::write)
    /// will always be within the range 0 through `ports` (exclusive).
    /// If there is a set amount of ports for this peripheral, make
    /// sure to throw an error here.
    fn init(ports: u8) -> anyhow::Result<Self>;

    /// Called when all connected ports have been dropped.
    /// 
    /// # Safety
    /// 
    /// Anything left hanging after this method is called
    /// (e.g. threads) will cause the emulator to crash.
    /// Make sure to `join()` all threads to the current
    /// one before or when this method is called.
    fn drop(self) {}

    /// Called every time a connected port is read from.
    /// 
    /// `port` is the exact connected port that was read from, and
    /// will always be within the range 0 through `ports` (exclusive),
    /// where `ports` is the `ports` argument in [`init`](Peripheral::init)
    fn read(&mut self, port: u8) -> u8 {
        0x00
    }

    /// Called every time a connected port is written to.
    /// 
    /// `port` is the exact connected port that was written to, and
    /// will always be within the range 0 through `ports` (exclusive),
    /// where `ports` is the `ports` argument in [`init`](Peripheral::init)
    fn write(&mut self, port: u8, data: u8) {}

    /// Called whenever the CPU in the emulator is reset.
    /// 
    /// Implementors should make sure that the peripheral state
    /// is reset to what it would be on [`init`](Peripheral::init).
    fn reset(&mut self) {}
}

/// Contains functions used for error handling and reporting.
pub mod errors {
    use std::{ cell::RefCell, thread_local, ffi::{c_int, c_char}};
    use anyhow::Error;

    thread_local! {
        static LAST_ERROR: RefCell<Option<Error>> = RefCell::new(None);
    }

    macro_rules! bail {
        ($ret:expr, $err:expr $(, $($arg:tt)*)?) => {
            $crate::errors::update_last_error(anyhow::anyhow!($err, $($arg)*));
            return $ret;
        };
    }
    
    /// Sets the last error to the provided error.
    pub fn update_last_error<E: Into<Error>>(err: E) {
        LAST_ERROR.with(|last| *last.borrow_mut() = Some(err.into()));
    }
    
    /// Gets the length of the last error message in bytes (*including* the **nul** byte).
    pub fn last_error_length() -> c_int {
        LAST_ERROR.with(|last| {
            last.borrow().as_ref().map(|err| err.to_string().len() + 1).unwrap_or(0)
        }) as c_int
    }
    
    fn get_error_message() -> Option<String> {
        LAST_ERROR.with(|last| last.borrow().as_ref().map(|err| err.to_string()))
    }
    
    /// Writes the last error message to the provided buffer,
    /// returning the number of bytes written.
    /// 
    /// Returns `0` if there is nothing to write.
    /// Returns `-1` if the function is given a null pointer.
    /// Returns `-2` if the buffer is not large enough for the error message.
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
            bail!(-2, "buffer not long enough for error message");
        }
    
        buffer[..error_message.len()].copy_from_slice(&error_message);
        buffer[error_message.len()] = b'\0';
    
        (error_message.len() + 1) as c_int
    }
}
