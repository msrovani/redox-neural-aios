//! Engines de voz — whisper.cpp (STT) + piper (TTS).

pub mod audio;
pub mod scheme_audio;
pub mod stt;
pub mod tts;

pub use audio::play_wav;
pub use scheme_audio::{capture_wav_scheme, scheme_enabled as audio_scheme_enabled};
pub use stt::{stt_from_env, SttEngine, SttKind};
pub use tts::{tts_from_env, TtsEngine, TtsKind, TtsOutput};
