//! Poll periódico de lifecycle agents no hermesd (ADR-011).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use hermes_core::event_client::EventClient;
use hermes_core::sgdb_client::SgdbClient;
use hermes_core::{run_lifecycle_cycle, run_self_heal};

/// Segundos entre ticks. `0` desliga. Default: 600 (SelfHeal) / 3600 (ciclo completo).
pub fn poll_secs() -> u64 {
    std::env::var("REDOX_LIFECYCLE_POLL_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(600)
}

pub fn full_cycle_every_n() -> u64 {
    std::env::var("REDOX_LIFECYCLE_FULL_EVERY")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(6)
}

/// Spawna thread de poll. Retorna flag para sinalizar stop (testes).
pub fn spawn_lifecycle_poll(events: EventClient, log: impl Fn(&str) + Send + 'static) -> Arc<AtomicBool> {
    let stop = Arc::new(AtomicBool::new(false));
    let stop_flag = Arc::clone(&stop);
    let secs = poll_secs();
    if secs == 0 {
        log("[hermesd] lifecycle poll desligado (REDOX_LIFECYCLE_POLL_SECS=0)");
        return stop;
    }
    let full_every = full_cycle_every_n().max(1);
    log(&format!(
        "[hermesd] lifecycle poll a cada {secs}s (full a cada {full_every} ticks)"
    ));

    thread::spawn(move || {
        let sgdb = SgdbClient::new();
        let mut tick_n = 0u64;
        loop {
            for _ in 0..secs {
                if stop_flag.load(Ordering::Relaxed) {
                    return;
                }
                thread::sleep(Duration::from_secs(1));
            }
            if stop_flag.load(Ordering::Relaxed) {
                return;
            }
            tick_n += 1;
            if tick_n % full_every == 0 {
                let out = run_lifecycle_cycle(&sgdb, &events);
                log(&format!(
                    "[hermesd] lifecycle full tick={tick_n}: {}",
                    out.lines().next().unwrap_or("ok")
                ));
            } else {
                let out = run_self_heal(&sgdb, &events);
                log(&format!(
                    "[hermesd] self_heal tick={tick_n}: {}",
                    out.lines().next().unwrap_or("ok")
                ));
            }
        }
    });

    stop
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn poll_secs_default_or_env() {
        let prev = std::env::var("REDOX_LIFECYCLE_POLL_SECS").ok();
        std::env::remove_var("REDOX_LIFECYCLE_POLL_SECS");
        assert_eq!(poll_secs(), 600);
        match prev {
            Some(v) => std::env::set_var("REDOX_LIFECYCLE_POLL_SECS", v),
            None => {}
        }
    }
}
