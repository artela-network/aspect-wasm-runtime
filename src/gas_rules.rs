use std::num::NonZeroU32;

use parity_wasm::elements::Instruction;
use wasm_instrument::gas_metering::{MemoryGrowCost, Rules};

pub struct GasRules;

impl Rules for GasRules {
    fn instruction_cost(&self, instruction: &Instruction) -> Option<u32> {
        use Instruction::*;
        let weight = match instruction {
            // These are taken from this post: https://github.com/paritytech/substrate/pull/7361#issue-506217103
            // from the table under the "Schedule" dropdown. Each decimal is multiplied by 10.
            // Note that those were calculated for wasi, not wasmtime, so they are likely very conservative.
            I64Const(_) => 40,
            I64Load(_, _) => 40,
            I64Store(_, _) => 80,
            Select => 150,
            If(_) => 150,
            Br(_) => 150,
            BrIf(_) => 150,
            BrTable(data) => 150 + (data.table.len() * 10) as u32,
            Call(_) => 950,
            // TODO: To figure out the param cost we need to look up the function
            CallIndirect(_, _) => 2000,
            GetLocal(_) => 40,
            SetLocal(_) => 80,
            TeeLocal(_) => 40,
            GetGlobal(_) => 60,
            SetGlobal(_) => 120,
            CurrentMemory(_) => 230,
            GrowMemory(_) => 435000,
            I64Clz => 850,
            I64Ctz => 850,
            I64Popcnt => 1080,
            I64ExtendSI32 => 20,
            I64ExtendUI32 => 20,
            I32WrapI64 => 20,
            I64Eqz => 26,
            I64Eq => 26,
            I64Ne => 26,
            I64LtS => 26,
            I64LtU => 26,
            I64GtS => 26,
            I64GtU => 26,
            I64LeS => 26,
            I64LeU => 26,
            I64GeS => 26,
            I64GeU => 26,
            I64Add => 160,
            I64Sub => 160,
            I64Mul => 200,
            I64DivS => 200,
            I64DivU => 200,
            I64RemS => 200,
            I64RemU => 200,
            I64And => 30,
            I64Or => 30,
            I64Xor => 30,
            I64Shl => 40,
            I64ShrS => 40,
            I64ShrU => 40,
            I64Rotl => 40,
            I64Rotr => 40,

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
            | I64Load32U(_, _) => 40,

            I32Store(_, _)
            | F32Store(_, _)
            | F64Store(_, _)
            | I32Store8(_, _)
            | I32Store16(_, _)
            | I64Store8(_, _)
            | I64Store16(_, _)
            | I64Store32(_, _) => 80,

            I32Const(_) | F32Const(_) | F64Const(_) => 10,
            I32Eqz => 13,
            I32Eq => 13,
            I32Ne => 13,
            I32LtS => 13,
            I32LtU => 13,
            I32GtS => 13,
            I32GtU => 13,
            I32LeS => 13,
            I32LeU => 13,
            I32GeS => 13,
            I32GeU => 13,
            I32Add => 80,
            I32Sub => 80,
            I32Mul => 100,
            I32DivS => 100,
            I32DivU => 100,
            I32RemS => 100,
            I32RemU => 100,
            I32And => 15,
            I32Or => 15,
            I32Xor => 15,
            I32Shl => 20,
            I32ShrS => 20,
            I32ShrU => 20,
            I32Rotl => 20,
            I32Rotr => 20,
            I32Clz => 470,
            I32Popcnt => 540,
            I32Ctz => 470,

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
            | F64Ne | F64Lt | F64Gt | F64Le | F64Ge | I32ReinterpretF32 | I64ReinterpretF64 => 500,
            F64Div | F64Sqrt | F32Div | F32Sqrt => 500,

            // More invented weights
            Block(_) => 150,
            Loop(_) => 150,
            Else => 150,
            End => 150,
            Return => 100,
            Drop => 100,
            SignExt(_) => 100,
            Nop => 10,
            Unreachable => 10,
        };
        Some(weight)
    }

    fn memory_grow_cost(&self) -> MemoryGrowCost {
        // a single page == 64KB
        // in EVM, the cost of growing memory is 3 gas per word (32 bytes) linear cost + words^2 / quad coefficient quadratic cost,
        // every page in EVM will cost: 6144 linear gas + 8192 quadratic gas = 14336 gas,
        // to convert this into wasm gas, we multiply the result by 1000, which is 14336000
        let gas_per_page =
            NonZeroU32::new(14336000).unwrap();

        MemoryGrowCost::Linear(gas_per_page)
    }

    fn call_per_local_cost(&self) -> u32 {
        return 0;
    }
}
