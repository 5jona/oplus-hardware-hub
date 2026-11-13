# OnePlus Smart FastCharge & Thermal Governor

A modular KernelSU extension for OnePlus devices running ColorOS or OxygenOS. Manages fast charging thermal throttling under active cooling to sustain peak wattage, featuring an automatic thermal tripwire and real-time KernelSU WebUI telemetry dashboard.

## Architecture

- **`governor/`**: Modern Rust 2024 workspace compiling an optimized native ARM64 daemon (`charge_governor`).
  - `governor-core`: Platform-agnostic telemetry reader, MMS `cool_down` interface, hysteresis state machine, and error handling.
  - `charge-governor`: Daemon supervisor and CLI status querying binary.
- **`webroot/`**: Responsive telemetry dashboard embedded in KernelSU Manager via `window.ksu.exec`.
- **`config.conf`**: Configurable temperature ceilings, recovery hysteresis points, and screen-on policy.

## Build & Installation

### Requirements
- Rust toolchain (Rust 2024 / 1.89+)
- Android NDK LLVM toolchain for `aarch64-linux-android`

### Building the Flashable ZIP
```bash
./build.sh
```
The output package will be generated at `out/oplus-smart-charge.zip`.

### Live Testing & Hot Reloading (via ADB)
```bash
./reload.sh 10.0.0.12:5555
```

## License
MIT OR Apache-2.0
