# Fateful Peripheral
![GitHub Workflow Status (with event)](https://img.shields.io/github/actions/workflow/status/commonkestrel/fateful_peripheral/rust.yml)

This is a utility library for working with [`fateful`](https://github.com/commonkestrel/fateful)'s peripheral system.
This allows for the easy creation of stateful peripherals while avoiding `unsafe` code wherever possible.

Stateful peripherals allow for shared global state wihout `unsafe` code.
This is achieved through a trait that contains all the functionality of a peripheral.


# Project Setup

In order to be dynamically loaded,
your project must be configured as a library.

In your `Cargo.toml`, add these lines:
```toml
[lib]
crate-type = ["rlib", "cdylib"]
```

This crate is not yet on `crates.io`,
so to add it to your project,
add this line under `[dependencies]`:
```toml
fateful_peripheral = { git = "https://github.com/commonkestrel/fateful_peripheral" }
```

# Examples

See an example of a simple peripheral that just acts as an extra register:

```rust
use fateful_peripheral::{ Peripheral, peripheral };
use anyhow::Result;

#[peripheral]
struct State {
    data: u8,
}

impl Peripheral for State {
    fn init(n: u8) -> Result<Self> {
        State {data: 0}
    }

    fn read(&mut self, n: u8) -> u8 {
        self.data
    }

    fn write(&mut self, n: u8, data: u8) {
        self.data = data;
    }

    fn reset(&mut self) {
        self.data = 0;
    }
}
```

As you can see, stateful peripherals are quite a bit safer than manually handling the FFI,
and follow Rust's conventions much more closesly.

The `perihperal` attribute also adds an easy way to add a name to a peripherals,
by setting the `name` property to a byte string:

```rust
#[peripheral(name = b"Example")]
struct State {
    /* ... */
}
```
