
# Aspect WASM Runtime Library

The Aspect WASM Runtime Library, is WASM execution runtime to run Artela Aspects.

## Supported Platforms

The library is currently compatible with multiple operating systems across different architectures:

- **Linux**: Arm64, x86_64
- **MacOS**: Arm64, x86_64
- **Windows**: x86_64

## Functionality

This library meticulously parses a given WASM file, injecting a gas counter as a global variable. Additionally, it integrates gas metering code into each designated code section, positioning it at the outset of the section body.

Furthermore, the library facilitates the export of both the gas counter and the start section of the WASM bytecode. These are exported as `__gas_counter__` and `__aspect_start__`, respectively. It is crucial to initialize the gas counter prior to executing the start section to avoid execution failures.

## Building the Library

To compile for your current platform, run:

```bash
make build
```

For compiling across all supported platforms, ensure the respective cross-compilers are installed:

```bash
make all
```

For a specific platform build, such as Darwin Arm64, execute:

```bash
make darwin-aarch64
```

Build artifacts are located in `target/{platform}/release` directory, with a dedicated directory `target/release` for the current platform build.

### Available Libraries

The build process generates three types of libraries to suit various project requirements:

- **Static Library**: `libaspect_wasm_instrument.a`
- **Dynamic Library**: `libaspect_wasm_instrument.so`, `libaspect_wasm_instrument.dylib`, `libaspect_wasm_instrument.dll`
- **Rust Library**: `aspect_wasm_instrument.rlib`

## Current Limitations

Some features are not yet supported due to the constraints of the underlying `wasm-instrument` library, including but not limited to:

- Bulk memory operations
- Reference types
- ...

Stay tuned for future updates as we continue to enhance this library's capabilities and compatibility.
