use wasmparser::{FuncValidatorAllocations, Parser, Payload, Validator, ValidPayload, WasmFeatures};
use wasmtime_c_api::wasmtime_error_t;

#[no_mangle]
pub unsafe extern "C" fn aspect_validate(
    wasm: *const u8,
    len: usize,
) -> Option<Box<wasmtime_error_t>> {
    let binary = slice_from_raw_parts(wasm, len);
    handle_result(validate(binary), |()| {})
}

fn validate(binary: &[u8]) -> anyhow::Result<()> {
    let mut validator = Validator::new_with_features(WasmFeatures {
        mutable_global: true,
        saturating_float_to_int: false,
        sign_extension: true,
        reference_types: false,
        multi_value: true,
        bulk_memory: false,
        simd: false,
        relaxed_simd: false,
        threads: false,
        tail_call: false,
        floats: false,
        multi_memory: false,
        exceptions: false,
        memory64: false,
        extended_const: false,
        component_model: false,
        function_references: false,
        memory_control: false,
        gc: false,
        component_model_values: false,
        component_model_nested_names: false,
    });

    let mut functions_to_validate = Vec::new();
    let mut entry_exported = false;
    for payload in Parser::new(0).parse_all(binary) {
        match payload {
            Ok(Payload::ExportSection(reader)) => {
                for export_result in reader {
                    let export = export_result?;
                    if export.name == "__aspect_start__" && matches!(export.kind, wasmparser::ExternalKind::Func) {
                        entry_exported = true;
                    }
                }
            }
            _ => {
                match validator.payload(&payload?)? {
                    ValidPayload::Func(a, b) => {
                        functions_to_validate.push((a, b));
                    }
                    _ => {}
                }
            }
        }
    }

    if !entry_exported {
        return Err(anyhow::anyhow!("Entrypoint function not exported"));
    }

    let mut allocs = FuncValidatorAllocations::default();
    for (func, body) in functions_to_validate {
        let mut validator = func.into_validator(allocs);
        validator.validate(&body)?;
        allocs = validator.into_allocations();
    }

    Ok(())
}

fn handle_result<T>(
    result: anyhow::Result<T>,
    ok: impl FnOnce(T),
) -> Option<Box<wasmtime_error_t>> {
    match result {
        Ok(value) => {
            ok(value);
            None
        }
        Err(error) => Some(Box::new(wasmtime_error_t::from(error))),
    }
}

/// Helper for creating Rust slices from C inputs.
///
/// This specifically disregards the `ptr` argument if the length is zero. The
/// `ptr` in that case maybe `NULL` or invalid, and it's not valid to have a
/// zero-length Rust slice with a `NULL` pointer.
unsafe fn slice_from_raw_parts<'a, T>(ptr: *const T, len: usize) -> &'a [T] {
    if len == 0 {
        &[]
    } else {
        std::slice::from_raw_parts(ptr, len)
    }
}

#[cfg(test)]
mod tests {
    use crate::validate::validate;

    #[test]
    fn test_validate_valid_wasm_bytes() {
        let wasm_path = "testdata/runtime_test.wasm";
        let wasm_bytes = std::fs::read(wasm_path).expect("Unable to read the wasm file");

        validate(&wasm_bytes).expect("Failed to validate the wasm file");
    }
}
