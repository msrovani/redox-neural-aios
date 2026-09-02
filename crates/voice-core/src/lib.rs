//! Voice pipeline — wake → STT → Hermes → TTS (Redox AIOS).

pub mod barge_in;
pub mod engines;
pub mod event_client;
pub mod hermes_client;
pub mod pipeline;
pub mod stub;

pub use barge_in::{barge_in_enabled, request_cancel, vad_active};
pub use engines::{
    capture_wav_scheme, play_wav, stt_from_env, tts_from_env, audio_scheme_enabled, SttKind,
    TtsKind, TtsOutput,
};
pub use event_client::{EventClient, DEFAULT_EVENTD_SOCKET};
pub use hermes_client::{HermesClient, DEFAULT_HERMES_SOCKET};
pub use pipeline::{VoicePipeline, VoiceResult};
pub use stub::{StubStt, StubTts, StubWakeWord};

pub const TOPIC_VOICE_WAKE: &str = "VOICE_WAKE";
pub const TOPIC_VOICE_STT: &str = "VOICE_STT";
pub const TOPIC_VOICE_TTS_START: &str = "VOICE_TTS_START";
pub const TOPIC_VOICE_TTS_END: &str = "VOICE_TTS_END";
pub const DEFAULT_VOICE_SOCKET: &str = "127.0.0.1:7744";
