use fateful_peripheral::{peripheral, Peripheral};
use anyhow::{Result, bail};

#[peripheral(name = b"Stateful Example")]
struct State {
    data: u8,
}

impl Peripheral for State {    
    fn init(n: u8) -> Result<Self> {
        if n != 1 {
            bail!("invalid number of ports: expected `1`, found `{n}`");
        }
        Ok(State { data: 0 })
    }

    fn read(&mut self, _n: u8) -> u8 {
        self.data
    }

    fn write(&mut self, _n: u8, data: u8) {
        self.data = data;
    }

    fn reset(&mut self) {
        self.data = 0;
    }
}
