# Hexapod PWM Gadget USB Server

## Introduction
This repository contains the USB server component of the Hexapod PWM Gadget firmware, which manages USB communication between the Hexapod device and a host computer. The project is developed in Rust and is intended for developers and enthusiasts working with the Hexapod PWM Gadget.

## Prerequisites
- Rust toolchain (cargo, rustc)
- Linux system with gadgetfs support
- Hexapod PWM Gadget hardware

## Installation and Building Instructions
1. **Clone the repository:**
   ```
   git clone https://github.com/HexapodBionik/hpg-server.git
   cd hpg-server
   ```
2. **Build the project:**
   ```
   cargo build --release
   ```
   The resulting executable will be located in the `target/release/` directory.
3. **Run the `setup-gadgetfs.sh` script to prepare your system for the USB server:**
   ```
   ./setup-gadgetfs.sh
   ```

## Usage
After building the project and running the setup script, you can start the server with the following command:
```
cargo run --release -- <USB device path> [optional filenames]
```
Replace `<USB device path>` with the path to your USB device.

## Configuration
Currently, there are no user-modifiable configuration files or environmental variables. However, command-line arguments can be used when starting the server to specify file paths.

## Troubleshooting
If you encounter issues, ensure that you have the correct permissions to access USB devices and that the gadgetfs kernel module is loaded. For additional help, please submit an issue on the GitHub repository.

## Contributing
Contributions to the `hpg-server` project are welcome! If you want to contribute code, documentation, or report bugs, please refer to the CONTRIBUTING.md file for guidelines.

## License
This project is released under the GPL-2.0 license. For more information, please see the LICENSE file in the repository.

## Contact
For support or to get in touch with the maintainer, you can open an issue on the GitHub repository or contact the author, Antoni Przybylik, directly through GitHub.
```
