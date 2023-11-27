# Fateful Peripheral

This is a utility library for working with [`fateful`](https://github.com/commonkestrel/fateful)'s peripheral system.
This allows for the easy creation of peripherals while avoiding `unsafe` code wherever possible.

# Stateless

Stateless peripherals are a quick and dirty method of making peripherals,
by using global variables instead of a shared state.
This, however, makes larger or more complicated peripherals very difficult,
and requires lots of `unsafe` code.
This also limits global state to variables that are `Send + Sync`.

See an example of a simple stateless peripheral that just acts as an extra register:

```rs
use fateful_peripheral::{read, write};

static mut STATE: u8 = 0;

#[read]
unsafe fn read(_: u8) -> u8 {
    STATE
}

#[write]
unsafe fn write(_: u8, data: u8) {
    STATE = data;
}
```

# Stateful

Stateful peripherals allow for shared global state wihout `unsafe` code.
This is achieved through a trait that contains all the functionality of a peripheral.

Here's the same peripheral, implemented with stateful logic:

```rs
use fateful_peripheral::{ Peripheral, peripheral };

#[peripheral]
struct State {
    data: u8,
}

impl Peripheral for State {
    fn init(n: u8) -> Self {
        State {data: 0}
    }

    fn read(&mut self, n: u8) -> u8 {
        self.data
    }

    fn write(&mut self, n: u8, data: u8) {
        self.data = data;
    }
}
```

As you can see, stateful peripherals are quite a bit safer,
and follow Rust's conventions much more closesly.

The `perihperal` attribute also adds an easy way to add a name to a peripherals,
by setting the `name` property to a byte string:

```rs
#[peripheral(name = b"Example")]
struct State {
    /* ... */
}
```
