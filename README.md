# Aspect WASM Instrument

This is a simple lib powered by `wasm-instrument` from Parity to instrument Artela Aspect WebAssembly (WASM) files to add gas metering and corresponding validation.

## Supported platforms

Currently, we provide support for the following platforms:

- Linux: Arm64, x86_64
- MacOS: Arm64, x86_64
- Windows: x86_64

## How it works

The lib will parse the given WASM file and inject the gas counter as a global variable and the gas metering code into each metered code section. The gas metering code will be added to the beginning of the section body.

This lib will also export the gas counter and the start section of the WASM bytecode.

The gas counter is exported as `__gas_counter__` and the start section is exported as `__aspect_start__`.

Make sure you initialize the gas counter before running start section, otherwise your execution will fail.

The code before wasm instrument is like:

```bash
# original wasm code
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
The code after wasm instrument is like: 

```bash
(func (;67;) (type 1) (param i32) (result i32)
    # How much gas to consume for the following section
    i64.const 11170
    # Consume gas
    call 30
    # execute the original code
    local.get 0
    local.get 0
    i64.extend_i32_u
    i64.const 50862630
    i64.mul
    call 66
    memory.grow
)

# The gas metering code
(func (;30;) (type 9) (param i64)
    global.get 5
    local.get 0
    i64.ge_u
    if ;; label = @1
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

# global gas counter
(global (;5;) (mut i64) i64.const 0)
# exported start section
(export "__aspect_start__" (func $~start))
# alias for gas counter
(export "__gas_counter__" (global 5))
```

## How to build

You can execute the following command to build for the current platform:

```bash
make build
```

Or you can build for all platforms (but make sure you have installed corresponding cross-compilers):

```bash 
make all
```

Or just for specific platform, for example:

```bash
make darwin-aarch64
```

The build artifact will the generated in the `target/{platform}/release` directory (for current platform, it will be located at `target/release` directory).

The generated artifact will provide three libs for you, pick the one that is suitable for your project:
- Static library: `libaspect_wasm_instrument.a`
- Dynamic library: `libaspect_wasm_instrument.so` / `libaspect_wasm_instrument.dylib` / `libaspect_wasm_instrument.dll`
- Rust Library: `aspect_wasm_instrument.rlib`

## Features not supported yet

Due to the limitation of the `wasm-instrument` lib, the following features are not supported yet:
- Bulk memory
- Reference types
- ...

