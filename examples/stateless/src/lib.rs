use fateful_peripheral::{read, write};

static mut STATE: u8 = 0;

#[read]
unsafe fn read(_n: u8) -> u8 {
    STATE
}

#[write]
unsafe fn write(_n: u8, data: u8) {
    STATE = data;
}
