#[derive(Clone, Copy, Debug)]
pub enum WhisperModelSource {
    Tiny,
    TinyEn,
    Base,
    BaseEn,
    Small,
    SmallEn,
    Medium,
    MediumEn,
    Large,
    LargeV2,
}

impl WhisperModelSource {
    pub fn is_multilingual(&self) -> bool {
        match self {
            Self::Tiny | Self::Base | Self::Small | Self::Medium | Self::Large | Self::LargeV2 => {
                true
            }
            Self::TinyEn | Self::BaseEn | Self::SmallEn | Self::MediumEn => false,
        }
    }

    pub(crate) fn model_and_revision(&self) -> (&'static str, &'static str) {
        match self {
            Self::Tiny => ("openai/whisper-tiny", "main"),
            Self::TinyEn => ("openai/whisper-tiny.en", "refs/pr/15"),
            Self::Base => ("openai/whisper-base", "refs/pr/22"),
            Self::BaseEn => ("openai/whisper-base.en", "refs/pr/13"),
            Self::Small => ("openai/whisper-small", "main"),
            Self::SmallEn => ("openai/whisper-small.en", "refs/pr/10"),
            Self::Medium => ("openai/whisper-medium", "main"),
            Self::MediumEn => ("openai/whisper-medium.en", "main"),
            Self::Large => ("openai/whisper-large", "refs/pr/36"),
            Self::LargeV2 => ("openai/whisper-large-v2", "refs/pr/57"),
        }
    }
}
