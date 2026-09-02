//! Backend scheme `memory:` nativo — Fase 2 ADR-005.
//! URI bridge (`open/`) quando `REDOX_MEMORY_SCHEME_NATIVE=1`; senão file bridge JSON.

pub fn scheme_native_enabled() -> bool {    std::env::var("REDOX_MEMORY_SCHEME_NATIVE")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or_else(|_| {
            // Target Redox: preferir scheme por default (factory.toml / init.d).
            std::env::var("REDOX_OS_TARGET")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("redox"))
                .unwrap_or(false)
        })
}

pub fn backend_label() -> &'static str {
    if scheme_native_enabled() {
        "scheme_uri_bridge"
    } else {
        "scheme_file_bridge"
    }
}