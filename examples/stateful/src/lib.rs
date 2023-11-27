use fateful_peripheral::{peripheral, Peripheral};

#[peripheral(name = b"Stateful Example")]
struct State {
    data: u8,
}

impl Peripheral for State {
    fn init(_n: u8) -> Self {
        State { data: 0 }
    }

    fn read(&mut self, _n: u8) -> u8 {
        self.data
    }

    fn write(&mut self, _n: u8, data: u8) {
        self.data = data;
    }
}
