#![doc = include!("../README.md")]

pub use macros::*;

#[allow(unused_variables)]
pub trait Peripheral: Sized {
    fn init(n: u8) -> Self;
    fn drop(self) {}

    fn read(&mut self, n: u8) -> u8 {
        0x00
    }
    fn write(&mut self, n: u8, data: u8) {}
}
