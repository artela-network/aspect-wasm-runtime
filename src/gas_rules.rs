use std::num::NonZeroU32;

use parity_wasm::elements::Instruction;
use wasm_instrument::gas_metering::{MemoryGrowCost, Rules};

/// Using 10 gas = ~1ns for WASM instructions.
const GAS_PER_SECOND: u64 = 10_000_000_000;
pub const CONST_MAX_GAS: u64 = 1000 * GAS_PER_SECOND;

pub struct GasRules;

impl Rules for GasRules {
    fn instruction_cost(&self, instruction: &Instruction) -> Option<u32> {
        use Instruction::*;
        let weight = match instruction {
            // These are taken from this post: https://github.com/paritytech/substrate/pull/7361#issue-506217103
            // from the table under the "Schedule" dropdown. Each decimal is multiplied by 10.
            // Note that those were calculated for wasi, not wasmtime, so they are likely very conservative.
            I64Const(_) => 2,
            I64Load(_, _) => 2,
            I64Store(_, _) => 4,
            Select => 10,
            If(_) => 10,
            Br(_) => 10,
            BrIf(_) => 10,
            BrTable(data) => 15 + data.table.len() as u32,
            Call(_) => 95,
            // TODO: To figure out the param cost we need to look up the function
            CallIndirect(_, _) => 200,
            GetLocal(_) => 2,
            SetLocal(_) => 2,
            TeeLocal(_) => 2,
            GetGlobal(_) => 2,
            SetGlobal(_) => 2,
            CurrentMemory(_) => 23,
            GrowMemory(_) => 435000,
            I64Clz => 85,
            I64Ctz => 85,
            I64Popcnt => 108,
            I64Eqz => 2,
            I64ExtendSI32 => 2,
            I64ExtendUI32 => 2,
            I32WrapI64 => 2,
            I64Eq => 2,
            I64Ne => 2,
            I64LtS => 2,
            I64LtU => 2,
            I64GtS => 2,
            I64GtU => 2,
            I64LeS => 2,
            I64LeU => 2,
            I64GeS => 2,
            I64GeU => 2,
            I64Add => 2,
            I64Sub => 2,
            I64Mul => 4,
            I64DivS => 8,
            I64DivU => 8,
            I64RemS => 8,
            I64RemU => 8,
            I64And => 2,
            I64Or => 2,
            I64Xor => 2,
            I64Shl => 4,
            I64ShrS => 4,
            I64ShrU => 4,
            I64Rotl => 2,
            I64Rotr => 2,

            // These are similar enough to something above so just referencing a similar
            // instruction
            I32Load(_, _)
            | F32Load(_, _)
            | F64Load(_, _)
            | I32Load8S(_, _)
            | I32Load8U(_, _)
            | I32Load16S(_, _)
            | I32Load16U(_, _)
            | I64Load8S(_, _)
            | I64Load8U(_, _)
            | I64Load16S(_, _)
            | I64Load16U(_, _)
            | I64Load32S(_, _)
            | I64Load32U(_, _) => 2,

            I32Store(_, _)
            | F32Store(_, _)
            | F64Store(_, _)
            | I32Store8(_, _)
            | I32Store16(_, _)
            | I64Store8(_, _)
            | I64Store16(_, _)
            | I64Store32(_, _) => 4,

            I32Const(_) | F32Const(_) | F64Const(_) => 1,
            I32Eqz => 1,
            I32Eq => 1,
            I32Ne => 1,
            I32LtS => 1,
            I32LtU => 1,
            I32GtS => 1,
            I32GtU => 1,
            I32LeS => 1,
            I32LeU => 1,
            I32GeS => 1,
            I32GeU => 1,
            I32Add => 1,
            I32Sub => 1,
            I32Mul => 2,
            I32DivS => 4,
            I32DivU => 4,
            I32RemS => 4,
            I32RemU => 4,
            I32And => 1,
            I32Or => 1,
            I32Xor => 1,
            I32Shl => 2,
            I32ShrS => 2,
            I32ShrU => 2,
            I32Rotl => 1,
            I32Rotr => 1,
            I32Clz => 47,
            I32Popcnt => 54,
            I32Ctz => 47,

            // Float weights not calculated by reference source material. Making up
            // some conservative values. The point here is not to be perfect but just
            // to have some reasonable upper bound.
            F64ReinterpretI64 | F32ReinterpretI32 | F64PromoteF32 | F64ConvertUI64
            | F64ConvertSI64 | F64ConvertUI32 | F64ConvertSI32 | F32DemoteF64 | F32ConvertUI64
            | F32ConvertSI64 | F32ConvertUI32 | F32ConvertSI32 | I64TruncUF64 | I64TruncSF64
            | I64TruncUF32 | I64TruncSF32 | I32TruncUF64 | I32TruncSF64 | I32TruncUF32
            | I32TruncSF32 | F64Copysign | F64Max | F64Min | F64Mul | F64Sub | F64Add
            | F64Trunc | F64Floor | F64Ceil | F64Neg | F64Abs | F64Nearest | F32Copysign
            | F32Max | F32Min | F32Mul | F32Sub | F32Add | F32Nearest | F32Trunc | F32Floor
            | F32Ceil | F32Neg | F32Abs | F32Eq | F32Ne | F32Lt | F32Gt | F32Le | F32Ge | F64Eq
            | F64Ne | F64Lt | F64Gt | F64Le | F64Ge | I32ReinterpretF32 | I64ReinterpretF64 => 50,
            F64Div | F64Sqrt | F32Div | F32Sqrt => 50,

            // More invented weights
            Block(_) => 10,
            Loop(_) => 10,
            Else => 10,
            End => 10,
            Return => 10,
            Drop => 10,
            SignExt(_) => 10,
            Nop => 1,
            Unreachable => 1,
        };
        Some(weight)
    }

    fn memory_grow_cost(&self) -> MemoryGrowCost {
        // Each page is 64KiB which is 65536 bytes.
        const PAGE: u64 = 64 * 1024;
        // 1 GB
        const GIB: u64 = 1073741824;
        // 12GiB to pages for the max memory allocation
        // In practice this will never be hit unless we also
        // free pages because this is 32bit WASM.
        const MAX_PAGES: u64 = 12 * GIB / PAGE;
        let gas_per_page =
            NonZeroU32::new((CONST_MAX_GAS / MAX_PAGES).try_into().unwrap()).unwrap();

        MemoryGrowCost::Linear(gas_per_page)
    }

    fn call_per_local_cost(&self) -> u32 {
        return 0;
    }
}
