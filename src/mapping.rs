use std::ffi::c_void;
use anyhow::anyhow;
use lazy_static::lazy_static;
use parity_wasm::elements::ExportEntry;
use slog::{Logger, o, warn};
use wasm_instrument::gas_metering::mutable_global;

use crate::gas_rules::GasRules;

lazy_static! {
    static ref LOGGER: Logger = Logger::root(slog::Discard, o!());
}

#[repr(C)]
pub struct WasmInstrumentResult {
    pub ptr: *mut c_void,
    pub len: usize,
}

#[no_mangle]
pub extern "C" fn wasm_instrument(raw_module: *const u8, len: usize) -> WasmInstrumentResult {
    let raw_module = unsafe { std::slice::from_raw_parts(raw_module, len) };
    match gas_metering_inject(&LOGGER, raw_module) {
        Ok(vec) => {
            let out = vec.into_boxed_slice();
            let len = out.len();
            let ptr = Box::into_raw(out) as *mut c_void;
            WasmInstrumentResult { ptr, len }
        },
        Err(_) => WasmInstrumentResult { ptr: std::ptr::null_mut(), len: 0 },
    }
}

#[no_mangle]
pub extern "C" fn wasm_instrument_free(ptr: *mut c_void) {
    unsafe { let _ = Box::from_raw(ptr as *mut u8); }
}

pub fn gas_metering_inject(
    logger: &Logger,
    raw_module: &[u8],
) -> Result<Vec<u8>, anyhow::Error> {
    // Add the gas calls here. Module name "gas" must match. See also
    // e3f03e62-40e4-4f8c-b4a1-d0375cca0b76. We do this by round-tripping the module through
    // parity - injecting gas then serializing again.
    let parity_module = parity_wasm::elements::Module::from_bytes(raw_module)?;
    let mut parity_module = match parity_module.parse_names() {
        Ok(module) => module,
        Err((errs, module)) => {
            for (index, err) in errs {
                warn!(
                        logger,
                        "unable to parse function name for index {}: {}",
                        index,
                        err.to_string()
                    );
            }

            module
        }
    };

    parity_module.start_section().map(|index| {
        let name = "__aspect_start__";

        parity_module.clear_start_section();
        parity_module
            .export_section_mut()
            .unwrap()
            .entries_mut()
            .push(ExportEntry::new(
                name.parse().unwrap(),
                parity_wasm::elements::Internal::Function(index),
            ));
    });
    let parity_module = wasm_instrument::gas_metering::inject(parity_module, mutable_global::Injector::new("__gas_counter__"), &GasRules)
        .map_err(|_| anyhow!("Failed to inject gas counter"))?;
    let raw_module = parity_module.into_bytes()?;

    Ok(raw_module)
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;

    use slog::{Logger, o};

    use crate::mapping::gas_metering_inject;

    #[test]
    fn test_wasm_to_wat() {
        let wasm_path = "../testdata/runtime_test.wasm";
        let wat_path = "output.wat";
        let injected_wasm_path = "output.wasm";

        let wasm_bytes = std::fs::read(wasm_path).expect("Unable to read the wasm file");

        let injected = gas_metering_inject(&Logger::root(slog::Discard, o!()),
                              &wasm_bytes).expect("Failed to create valid module");

        let mut file = File::create(injected_wasm_path).expect("Unable to create wasm file");
        file.write_all(&injected).expect("Unable to write to wasm file");

        let wat_string = wasmprinter::print_bytes(injected).expect("Failed to convert wasm to wat");

        let mut file = File::create(wat_path).expect("Unable to create wat file");
        file.write_all(wat_string.as_bytes()).expect("Unable to write to wat file");
    }
}
