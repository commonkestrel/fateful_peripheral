use fateful_peripheral::{peripheral, Peripheral};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
enum Error {
    InvalidN,
}

#[peripheral(name = b"Stateful Example")]
struct State {
    data: u8,
}

impl Peripheral for State {
    type Err = Error;

    fn init(n: u8) -> Result<Self, Error> {
        if n != 1 {
            return Err(Error::InvalidN);
        }
        Ok(State { data: 0 })
    }

    fn read(&mut self, _n: u8) -> u8 {
        self.data
    }

    fn write(&mut self, _n: u8, data: u8) {
        self.data = data;
    }
}
