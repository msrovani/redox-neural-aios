//! Execução sandbox wasmi — Caminho A ADR-010.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use wasmi::{Config, Engine, Error, Linker, Module, Store};

use crate::caps::{Cap, CAP_FS, CAP_LOG, CAP_NET};

pub const DEFAULT_FUEL: u64 = 5_000_000;

static TICKS: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub struct WasmError(pub String);

impl From<&'static str> for WasmError {
    fn from(v: &'static str) -> Self {
        Self(v.into())
    }
}

impl From<Error> for WasmError {
    fn from(e: Error) -> Self {
        Self(e.to_string())
    }
}

pub struct HostState {
    pub caps: Cap,
    pub log: Vec<u8>,
}

impl HostState {
    fn new(caps: Cap) -> Self {
        Self {
            caps,
            log: Vec::new(),
        }
    }
}

fn check_cap(state: &HostState, required: Cap) -> Result<(), Error> {
    if state.caps & required == 0 {
        return Err(Error::new("capability denied"));
    }
    Ok(())
}

fn read_wasm_str(caller: &wasmi::Caller<'_, HostState>, ptr: i32, len: i32) -> Result<String, Error> {
    let Some(wasmi::Extern::Memory(mem)) = caller.get_export("memory") else {
        return Err(Error::new("wasm: export memory ausente"));
    };
    let data = mem.data(caller);
    let p = ptr as usize;
    let l = (len as usize).min(4096);
    if p.saturating_add(l) > data.len() {
        return Err(Error::new("wasm: path fora de memory"));
    }
    String::from_utf8(data[p..p + l].to_vec()).map_err(|_| Error::new("wasm: path inválido"))
}

fn install_host_abi(linker: &mut Linker<HostState>) -> Result<(), WasmError> {
    linker
        .func_wrap(
            "aios",
            "log",
            |mut caller: wasmi::Caller<'_, HostState>, ptr: i32, len: i32| -> Result<(), Error> {
                check_cap(caller.data(), CAP_LOG)?;
                if let Some(wasmi::Extern::Memory(mem)) = caller.get_export("memory") {
                    let data = mem.data(&caller);
                    let p = ptr as usize;
                    let l = (len as usize).min(4096);
                    if p.saturating_add(l) <= data.len() {
                        let chunk = data[p..p + l].to_vec();
                        caller.data_mut().log.extend_from_slice(&chunk);
                    }
                }
                Ok(())
            },
        )
        .map_err(|_| "linker aios::log")?;

    linker
        .func_wrap("aios", "get_tick", |caller: wasmi::Caller<'_, HostState>| {
            check_cap(caller.data(), CAP_LOG)?;
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            TICKS.store(now, Ordering::Relaxed);
            Ok(now as i64)
        })
        .map_err(|_| "linker aios::get_tick")?;

    linker
        .func_wrap(
            "aios",
            "net_fetch",
            |caller: wasmi::Caller<'_, HostState>, _ptr: i32, _len: i32| -> Result<i32, Error> {
                check_cap(caller.data(), CAP_NET)?;
                if !agent_core::grant_active("net_fetch") {
                    return Err(Error::new("aios::net_fetch: grant net_fetch ausente no CapGate"));
                }
                Err(Error::new("aios::net_fetch gated — use host tools (REDOX_TOOLS_NET)"))
            },
        )
        .map_err(|_| "linker aios::net_fetch")?;

    linker
        .func_wrap(
            "aios",
            "fs_read",
            |caller: wasmi::Caller<'_, HostState>, path_ptr: i32, path_len: i32| -> Result<i32, Error> {
                check_cap(caller.data(), CAP_FS)?;
                if !agent_core::grant_active("fs_read") {
                    return Err(Error::new("aios::fs_read: grant fs_read ausente no CapGate"));
                }
                let path = read_wasm_str(&caller, path_ptr, path_len)?;
                let meta = std::fs::metadata(&path).map_err(|e| Error::new(e.to_string()))?;
                if !meta.is_file() {
                    return Err(Error::new("aios::fs_read: não é arquivo"));
                }
                Ok(meta.len().min(i32::MAX as u64) as i32)
            },
        )
        .map_err(|_| "linker aios::fs_read")?;

    Ok(())
}

fn validate_magic(wasm: &[u8]) -> Result<(), WasmError> {
    if wasm.len() < 8 || wasm[0..4] != [0x00, 0x61, 0x73, 0x6d] {
        return Err("wasm: magic inválido".into());
    }
    Ok(())
}

fn run_engine_call<R>(
    wasm: &[u8],
    func_name: &str,
    caps: Cap,
    invoke: impl FnOnce(
        &mut Store<HostState>,
        wasmi::TypedFunc<R, i32>,
    ) -> Result<i32, Error>,
) -> Result<i32, WasmError>
where
    R: wasmi::WasmTyList,
{
    validate_magic(wasm)?;
    let mut config = Config::default();
    config.consume_fuel(true);
    let engine = Engine::new(&config);
    let module = Module::new(&engine, wasm).map_err(WasmError::from)?;
    let mut store = Store::new(&engine, HostState::new(caps));
    store
        .set_fuel(DEFAULT_FUEL)
        .map_err(|e| WasmError(format!("set_fuel: {e}")))?;
    let mut linker = Linker::new(&engine);
    install_host_abi(&mut linker)?;
    let instance = linker
        .instantiate(&mut store, &module)
        .map_err(WasmError::from)?
        .start(&mut store)
        .map_err(WasmError::from)?;
    let func = instance
        .get_typed_func::<R, i32>(&store, func_name)
        .map_err(|_| WasmError("wasm: export não encontrado".into()))?;
    invoke(&mut store, func).map_err(WasmError::from)
}

pub fn run_i32_2(wasm: &[u8], func_name: &str, a: i32, b: i32, caps: Cap) -> Result<i32, WasmError> {
    run_engine_call(wasm, func_name, caps, |store, func| func.call(store, (a, b)))
}

pub fn run_i32_0(wasm: &[u8], func_name: &str, caps: Cap) -> Result<i32, WasmError> {
    run_engine_call(wasm, func_name, caps, |store, func| func.call(store, ()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::ADD_WASM;

    #[test]
    fn add_wasm_self_test() {
        let out = run_i32_2(ADD_WASM, "add", 2, 3, CAP_LOG).expect("add");
        assert_eq!(out, 5);
    }
}
