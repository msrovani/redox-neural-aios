//! Runtime WASM Caminho A (ADR-010) — wasmi + fuel + host ABI mínimo.

mod caps;
mod modules;
mod op_ir;
mod runtime;

pub use caps::{Cap, CAP_LOG, CAP_NONE};
pub use modules::{add_module, echo_len_module, ADD_WASM};
pub use op_ir::{
    build_and_run_2, build_run_module, compile_expression, compile_return_literal, self_test,
    schema_hint, Op, ValType,
};
pub use runtime::{run_i32_0, run_i32_2, DEFAULT_FUEL, WasmError};
