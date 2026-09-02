//! EventBus pub/sub para daemons Redox AIOS (userspace, std).
//! Backend futuro: scheme `chan:` do Redox. Fase 0: in-process + Unix socket local.

pub mod bus;
pub mod chan;
pub mod client;
pub mod event;
pub mod remote;

pub use bus::{EventBus, Receiver};
pub use client::{EventClient, DEFAULT_EVENTD_SOCKET};
pub use event::{CapabilityToken, Event};
pub use remote::handle_remote;
