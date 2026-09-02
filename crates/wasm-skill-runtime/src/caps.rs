//! Capability bitmask — mapeamento futuro para scheme capabilities Redox.

pub type Cap = u32;

pub const CAP_LOG: Cap = 1 << 0;
pub const CAP_NET: Cap = 1 << 1;
pub const CAP_FS: Cap = 1 << 2;
pub const CAP_NONE: Cap = 0;
