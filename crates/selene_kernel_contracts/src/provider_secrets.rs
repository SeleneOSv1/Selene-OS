#![forbid(unsafe_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ProviderSecretId {
    BraveSearchApiKey,
    OpenAIApiKey,
    GoogleApiKey,
    GoogleSttApiKey,
    GoogleTtsApiKey,
    AzureSpeechKey,
    DeepgramApiKey,
    ElevenLabsApiKey,
    AnthropicApiKey,
}

impl ProviderSecretId {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BraveSearchApiKey => "brave_search_api_key",
            Self::OpenAIApiKey => "openai_api_key",
            Self::GoogleApiKey => "google_api_key",
            Self::GoogleSttApiKey => "google_stt_api_key",
            Self::GoogleTtsApiKey => "google_tts_api_key",
            Self::AzureSpeechKey => "azure_speech_key",
            Self::DeepgramApiKey => "deepgram_api_key",
            Self::ElevenLabsApiKey => "elevenlabs_api_key",
            Self::AnthropicApiKey => "anthropic_api_key",
        }
    }

    pub const fn all() -> &'static [Self] {
        &[
            Self::BraveSearchApiKey,
            Self::OpenAIApiKey,
            Self::GoogleApiKey,
            Self::GoogleSttApiKey,
            Self::GoogleTtsApiKey,
            Self::AzureSpeechKey,
            Self::DeepgramApiKey,
            Self::ElevenLabsApiKey,
            Self::AnthropicApiKey,
        ]
    }

    pub fn parse(raw: &str) -> Option<Self> {
        let normalized = raw.trim().to_ascii_lowercase();
        match normalized.as_str() {
            "brave_search_api_key" => Some(Self::BraveSearchApiKey),
            "openai_api_key" => Some(Self::OpenAIApiKey),
            "google_api_key" => Some(Self::GoogleApiKey),
            "google_stt_api_key" => Some(Self::GoogleSttApiKey),
            "google_tts_api_key" => Some(Self::GoogleTtsApiKey),
            "azure_speech_key" => Some(Self::AzureSpeechKey),
            "deepgram_api_key" => Some(Self::DeepgramApiKey),
            "elevenlabs_api_key" => Some(Self::ElevenLabsApiKey),
            "anthropic_api_key" => Some(Self::AnthropicApiKey),
            _ => None,
        }
    }

    pub fn allowed_key_names() -> Vec<&'static str> {
        Self::all().iter().map(|id| id.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::ProviderSecretId;

    #[test]
    fn provider_secret_ids_are_roundtrippable() {
        for secret in ProviderSecretId::all() {
            let parsed = ProviderSecretId::parse(secret.as_str());
            assert_eq!(parsed, Some(*secret));
        }
    }
}
