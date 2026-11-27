use std::sync::Arc;

use craft_ai_client::AiClient;
use craft_ai_core::payload::PayloadKind;
use craft_ai_core::{
    AppId, Auth, Billing, CapabilityId, Connection, ConnectionId, ConsentLedger, Disclosure, Event,
    InputRole, JobState, Locality, ModelId, Payload, Provider, Request, SecretString,
};
use craft_ai_openai::{OpenAiConfig, OpenAiProvider};
use grammarcraft_core::{AiConsentStatus, AiLocalityMode, AiMode, AiProposal, compute_word_diff};

use crate::CmdError;

const DEFAULT_CLOUD_ENDPOINT: &str = "https://api.openai.com/v1";
const DEFAULT_CLOUD_MODEL: &str = "gpt-4o-mini";
const MAX_SIMULATION_POLLS: usize = 256;

/// Backend abstraction for generating proposed text transformations.
pub trait AiBackend: Send + Sync + std::fmt::Debug {
    fn label(&self) -> &str;
    fn generate(&self, mode: AiMode, input_text: &str, detail: &str) -> Result<String, CmdError>;
}

/// Deterministic zero-network local text assistant for offline mode and fallback.
#[derive(Debug, Default)]
pub struct LocalOfflineBackend;

impl AiBackend for LocalOfflineBackend {
    fn label(&self) -> &str {
        "local-offline"
    }

    fn generate(&self, mode: AiMode, input_text: &str, detail: &str) -> Result<String, CmdError> {
        let trimmed = input_text.trim();
        match mode {
            AiMode::Rewrite => Ok(rewrite_offline(trimmed, detail)),
            AiMode::AdjustTone => Ok(adjust_tone_offline(trimmed, detail)),
            AiMode::Summarize => Ok(summarize_offline(trimmed, detail)),
            AiMode::Expand => Ok(expand_offline(trimmed, detail)),
            AiMode::Draft => Ok(draft_offline(detail)),
        }
    }
}

/// Adapter backed by any [`craft_ai_core::Provider`] (`OpenAiProvider` or `MockProvider`).
#[derive(Clone)]
pub struct CraftProviderBackend {
    label: String,
    model_id: ModelId,
    provider: Arc<dyn Provider>,
}

impl std::fmt::Debug for CraftProviderBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CraftProviderBackend")
            .field("label", &self.label)
            .field("model_id", &self.model_id)
            .finish()
    }
}

impl CraftProviderBackend {
    pub fn new(label: impl Into<String>, model_id: ModelId, provider: Arc<dyn Provider>) -> Self {
        Self {
            label: label.into(),
            model_id,
            provider,
        }
    }

    pub fn openai(base_url: &str, api_key: Option<String>, model: &str) -> Result<Self, CmdError> {
        let mut config = OpenAiConfig::new(base_url);
        if let Some(key) = api_key
            && !key.trim().is_empty()
        {
            config = config.with_key(SecretString::new(key));
        }
        let provider = OpenAiProvider::new(config)
            .map_err(|e| CmdError::Params(format!("invalid OpenAI config: {e}")))?;
        let model_id =
            ModelId::new(model).map_err(|e| CmdError::Params(format!("invalid model id: {e}")))?;
        Ok(Self::new("openai-compat", model_id, Arc::new(provider)))
    }
}

impl AiBackend for CraftProviderBackend {
    fn label(&self) -> &str {
        &self.label
    }

    fn generate(&self, mode: AiMode, input_text: &str, detail: &str) -> Result<String, CmdError> {
        let capability = CapabilityId::parse("text.complete")
            .map_err(|e| CmdError::Failed(format!("invalid capability id: {e}")))?;
        let instruction = system_instruction(mode, detail);
        let user_prompt = if input_text.trim().is_empty() {
            detail.to_owned()
        } else {
            format!("{instruction}\n\n{input_text}")
        };
        let req = Request::new(capability)
            .with_input(InputRole::System, Payload::text(instruction))
            .with_input(InputRole::Prompt, Payload::text(user_prompt));

        let job_id = self
            .provider
            .submit(&self.model_id, &req)
            .map_err(|e| CmdError::Failed(format!("AI submit failed: {e}")))?;

        let mut streamed = String::new();
        for _ in 0..MAX_SIMULATION_POLLS {
            let snap = self
                .provider
                .poll(&job_id)
                .map_err(|e| CmdError::Failed(format!("AI poll failed: {e}")))?;
            for event in snap.events {
                match event {
                    Event::TextDelta { text } => streamed.push_str(&text),
                    Event::Output(out) => {
                        if let Some(text) = out.payload.as_text() {
                            return Ok(text.to_owned());
                        }
                    }
                    Event::ToolCall { .. } | Event::Notice { .. } => {}
                }
            }
            match snap.state {
                JobState::Succeeded => return Ok(streamed),
                JobState::Failed { error } => {
                    return Err(CmdError::Failed(format!("AI provider error: {error}")));
                }
                JobState::Cancelled => {
                    return Err(CmdError::Failed("AI job was cancelled".to_owned()));
                }
                JobState::Queued | JobState::Running { .. } => {}
            }
        }
        Err(CmdError::Failed("AI job exceeded poll budget".to_owned()))
    }
}

/// Per-session AI locality, Craft [`ConsentLedger`], optional cloud backend, and staged proposal.
#[derive(Debug)]
pub struct AiSessionState {
    pub mode: AiLocalityMode,
    pub endpoint: String,
    pub consent_ledger: ConsentLedger,
    pub pending_proposal: Option<AiProposal>,
    next_proposal_id: u64,
    cloud_backend: Option<Arc<dyn AiBackend>>,
}

impl Default for AiSessionState {
    fn default() -> Self {
        Self {
            mode: AiLocalityMode::LocalOffline,
            endpoint: DEFAULT_CLOUD_ENDPOINT.to_owned(),
            consent_ledger: ConsentLedger::default(),
            pending_proposal: None,
            next_proposal_id: 1,
            cloud_backend: None,
        }
    }
}

impl AiSessionState {
    pub fn set_cloud_backend(&mut self, backend: Option<Arc<dyn AiBackend>>) {
        self.cloud_backend = backend;
    }

    pub fn set_cloud_provider(
        &mut self,
        model: &str,
        provider: Arc<dyn Provider>,
    ) -> Result<(), CmdError> {
        let model_id =
            ModelId::new(model).map_err(|e| CmdError::Params(format!("invalid model id: {e}")))?;
        self.cloud_backend = Some(Arc::new(CraftProviderBackend::new(
            "craft-provider",
            model_id,
            provider,
        )));
        Ok(())
    }

    pub fn disclosure_for(&self, input_bytes: usize) -> Result<Disclosure, CmdError> {
        let app_id = AppId::new("grammarcraft")
            .map_err(|e| CmdError::Failed(format!("invalid app id: {e}")))?;
        let conn_id = ConnectionId::new("openai-cloud")
            .map_err(|e| CmdError::Failed(format!("invalid connection id: {e}")))?;
        let cap_id = CapabilityId::parse("text.complete")
            .map_err(|e| CmdError::Failed(format!("invalid capability id: {e}")))?;

        let connection = Connection {
            id: conn_id,
            label: "GrammarCraft Cloud AI".to_owned(),
            provider: "openai".to_owned(),
            endpoint: Some(self.endpoint.clone()),
            locality: Locality::Remote,
            auth: Auth::None,
            billing: Billing::Metered,
            models: Vec::new(),
            enabled: true,
            key_header: None,
        };
        let sample = "x".repeat(input_bytes.min(16));
        let req = Request::new(cap_id).with_input(InputRole::Prompt, Payload::text(sample));
        let mut disclosure = Disclosure::for_request(&app_id, &connection, &req);
        if let Some(item) = disclosure.items.first_mut() {
            item.bytes = Some(input_bytes as u64);
        }
        Ok(disclosure)
    }

    pub fn consent_status(&self, input_bytes: usize) -> Result<AiConsentStatus, CmdError> {
        if self.mode == AiLocalityMode::LocalOffline {
            return Ok(AiConsentStatus::NotRequiredLocal);
        }
        let disclosure = self.disclosure_for(input_bytes)?;
        if self.consent_ledger.needs_consent(&disclosure, now_secs()) {
            Ok(AiConsentStatus::RequiredPending {
                destination: disclosure.destination,
                capability: disclosure.capability.to_string(),
                payload_bytes: input_bytes,
            })
        } else {
            Ok(AiConsentStatus::Granted {
                destination: disclosure.destination,
            })
        }
    }

    pub fn set_consent(&mut self, granted: bool) -> Result<AiConsentStatus, CmdError> {
        let disclosure = self.disclosure_for(0)?;
        if granted {
            self.consent_ledger.grant(&disclosure);
        } else {
            self.consent_ledger
                .revoke(&disclosure.app, &disclosure.connection);
        }
        self.consent_status(0)
    }

    pub fn launcher_client_available() -> bool {
        AppId::new("grammarcraft")
            .ok()
            .and_then(|id| AiClient::standard(id).ok())
            .is_some()
    }

    pub fn stage_proposal(
        &mut self,
        ai_mode: AiMode,
        original: &str,
        detail: &str,
    ) -> Result<AiProposal, CmdError> {
        let proposed = match self.mode {
            AiLocalityMode::LocalOffline => {
                LocalOfflineBackend.generate(ai_mode, original, detail)?
            }
            AiLocalityMode::CloudConsentRequired => {
                let disclosure = self.disclosure_for(original.len().max(detail.len()))?;
                if self.consent_ledger.needs_consent(&disclosure, now_secs()) {
                    return Err(CmdError::Disabled(format!(
                        "remote AI consent required before sending data to {}",
                        disclosure.destination
                    )));
                }
                let backend = self.cloud_backend.as_ref().ok_or_else(|| {
                    CmdError::Failed(format!(
                        "cloud AI consent granted for {}, but no remote provider/API key is configured (use Local (Offline) or configure OpenAI provider)",
                        disclosure.destination
                    ))
                })?;
                backend.generate(ai_mode, original, detail)?
            }
        };

        let provider_label = match self.mode {
            AiLocalityMode::LocalOffline => LocalOfflineBackend.label().to_owned(),
            AiLocalityMode::CloudConsentRequired => self
                .cloud_backend
                .as_ref()
                .map(|b| b.label().to_owned())
                .unwrap_or_else(|| DEFAULT_CLOUD_MODEL.to_owned()),
        };
        let hunks = compute_word_diff(original, &proposed)?;
        let proposal = AiProposal {
            id: self.next_proposal_id,
            mode: ai_mode,
            provider_label,
            original: original.to_owned(),
            proposed,
            hunks,
        };
        self.next_proposal_id = self.next_proposal_id.saturating_add(1);
        self.pending_proposal = Some(proposal.clone());
        Ok(proposal)
    }
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

fn system_instruction(mode: AiMode, detail: &str) -> String {
    match mode {
        AiMode::Rewrite => {
            if detail.trim().is_empty() {
                "Rewrite for clarity and conciseness while preserving meaning.".to_owned()
            } else {
                format!("Rewrite following instruction: {detail}")
            }
        }
        AiMode::AdjustTone => format!(
            "Adjust tone to `{}` while preserving factual content.",
            if detail.trim().is_empty() {
                "formal"
            } else {
                detail.trim()
            }
        ),
        AiMode::Summarize => "Summarize the document concisely.".to_owned(),
        AiMode::Expand => format!("Expand the draft clearly: {detail}"),
        AiMode::Draft => format!("Write a clear structured draft for: {detail}"),
    }
}

fn rewrite_offline(text: &str, detail: &str) -> String {
    let mut out = text
        .replace("due to the fact that", "because")
        .replace("in order to", "to")
        .replace("at this point in time", "now")
        .replace("very unique", "unique")
        .replace("a lot of", "many");
    if !detail.trim().is_empty() && out == text {
        out = format!("{} [Refined: {}]", collapse_spaces(text), detail.trim());
    } else {
        out = collapse_spaces(&out);
    }
    out
}

fn adjust_tone_offline(text: &str, tone: &str) -> String {
    let normalized = tone.trim().to_ascii_lowercase();
    if normalized == "friendly" || normalized == "cordial" {
        let base = collapse_spaces(text);
        if base.starts_with("Please note") {
            base
        } else {
            format!("Hi there — {base}")
        }
    } else {
        text.replace("don't", "do not")
            .replace("can't", "cannot")
            .replace("won't", "will not")
            .replace("gonna", "going to")
            .replace("wanna", "want to")
    }
}

fn summarize_offline(text: &str, detail: &str) -> String {
    let max_sentences = detail.trim().parse::<usize>().unwrap_or(2).clamp(1, 5);
    let sentences: Vec<&str> = text
        .split_inclusive(['.', '!', '?'])
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    if sentences.is_empty() {
        return collapse_spaces(text);
    }
    sentences
        .into_iter()
        .take(max_sentences)
        .collect::<Vec<_>>()
        .join(" ")
}

fn expand_offline(text: &str, detail: &str) -> String {
    let note = if detail.trim().is_empty() {
        "This builds on the core objective with clear, verifiable milestones."
    } else {
        detail.trim()
    };
    if text.is_empty() {
        note.to_owned()
    } else {
        format!("{text}\n\n{note}")
    }
}

fn draft_offline(prompt: &str) -> String {
    let topic = prompt.trim();
    if topic.is_empty() {
        "Draft overview: outline your key objective, supporting evidence, and next action."
            .to_owned()
    } else {
        format!(
            "{topic}\n\nKey points:\n- Context and objective for {topic}.\n- Evidence and clear recommendations.\n- Concrete next steps."
        )
    }
}

fn collapse_spaces(input: &str) -> String {
    input.split_whitespace().collect::<Vec<_>>().join(" ")
}

trait PayloadTextExt {
    fn as_text(&self) -> Option<&str>;
}

impl PayloadTextExt for Payload {
    fn as_text(&self) -> Option<&str> {
        if self.kind() != PayloadKind::Text {
            return None;
        }
        match self {
            Payload::Text { text } => Some(text.as_str()),
            _ => None,
        }
    }
}
