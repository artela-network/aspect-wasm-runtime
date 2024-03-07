
# Aspect WASM Instrumentation Library

The Aspect WASM Instrumentation Library, utilizing `wasm-instrument` from Parity, is designed to instrument Artela Aspect WebAssembly (WASM) files. It seamlessly adds gas metering functionality and ensures corresponding validation, enhancing the execution efficiency and reliability of your WebAssembly applications.

## Supported Platforms

The library is currently compatible with multiple operating systems across different architectures:

- **Linux**: Arm64, x86_64
- **MacOS**: Arm64, x86_64
- **Windows**: x86_64

## Functionality

This library meticulously parses a given WASM file, injecting a gas counter as a global variable. Additionally, it integrates gas metering code into each designated code section, positioning it at the outset of the section body.

Furthermore, the library facilitates the export of both the gas counter and the start section of the WASM bytecode. These are exported as `__gas_counter__` and `__aspect_start__`, respectively. It is crucial to initialize the gas counter prior to executing the start section to avoid execution failures.

### Before and After Instrumentation

- **Original WASM Code Snippet:**

```bash
# Original wasm code
(func (;67;) (type 1) (param i32) (result i32)
    local.get 0
    local.get 0
    i64.extend_i32_u
    i64.const 50862630
    i64.mul
    call 66
    memory.grow
)
```

- **Post-Instrumentation Code Snippet:**

```bash
(func (;67;) (type 1) (param i32) (result i32)
    # Gas consumption for the section
    i64.const 11170
    # Consuming gas
    call 30
    # Executing original code
    local.get 0
    local.get 0
    i64.extend_i32_u
    i64.const 50862630
    i64.mul
    call 66
    memory.grow
)

# Gas metering logic
(func (;30;) (type 9) (param i64)
    global.get 5
    local.get 0
    i64.ge_u
    if
      global.get 5
      local.get 0
      i64.sub
      global.set 5
    else
      i64.const -1
      global.set 5
      unreachable
    end
)

# Global gas counter
(global (;5;) (mut i64) i64.const 0)
# Exported start section
(export "__aspect_start__" (func $~start))
# Gas counter alias
(export "__gas_counter__" (global 5))
```

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
