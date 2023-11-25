# Hexapod PWM Gadget USB Server

Part of firmware for `Hexapod PWM Gadget` device.

# Building

To build the project run:
```
cargo build --release
```

Resulting executable file will be in
`target/release/` directory. You can
run it directly or through `cargo`
package manager by issuing the command:
```
cargo run --release -- <USB device path> {filenames}
```

# Usage

Before starting the server you should run
`setup-gadgetfs.sh` script.
