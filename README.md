# OPlus Hardware Hub

A modular KernelSU extension for OnePlus devices running ColorOS. Manages fast charging thermal throttling to sustain peak wattage under cooling, featuring an automatic thermal tripwire and real-time KernelSU WebUI telemetry dashboard.

## Architecture

- **`daemon/`**: Modern Rust 2024 workspace compiling an optimized native ARM64 binary (`ohh`).
  - `ohh-core`: Platform-agnostic telemetry reader, MMS `cool_down` interface, hysteresis state machine, and error handling.
  - `ohh`: Daemon supervisor and CLI status querying binary.
- **`webroot/`**: Responsive telemetry dashboard embedded in KernelSU Manager via `window.ksu.exec`.
- **`config.conf`**: Configurable temperature ceilings, recovery hysteresis points, screen-on policy, and ColorOS built-in custom charge limit controls.

## Key Features

- **Thermal Fast Charging Governor**: Sustains peak charging wattage by managing MMS cool_down gears, with automatic safety tripwire and hysteresis recovery.
- **ColorOS Custom Charge Limit Integration**: Directly integrates system `regular_charge_protection` switch and cutoff level (80% - 100%, 5% steps) with two-way synchronization between WebUI, `config.conf`, and ColorOS settings.
- **KernelSU WebUI Telemetry**: Real-time delivered power wattage, dual-cell voltage, battery temperature, and quick preset controls.

## Build & Installation

### Requirements
- Rust toolchain (Rust 2024 / 1.89+)
- Android NDK LLVM toolchain for `aarch64-linux-android`

### Building the Flashable ZIP
```bash
./build.sh
```
The output package will be generated at `out/oplus-hardware-hub.zip`.

### Live Testing & Hot Reloading (via ADB)
```bash
./reload.sh 10.0.0.12:5555
```
