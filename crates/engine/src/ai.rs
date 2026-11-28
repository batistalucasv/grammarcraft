use std::fmt::Write as _;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use craft_ai_client::{
    AiClient, EngineRegistry, LoadedModel, LoadedModelInfo, ModelRunner, RunContext, Unloaded,
};
use craft_ai_core::history::{
    EndRecord, HISTORY_SCHEMA, HistoryEntry, Outcome, StartRecord, merge,
};
use craft_ai_core::payload::PayloadKind;
use craft_ai_core::{
    AppId, Auth, Billing, CapabilityId, Connection, ConnectionId, ConsentLedger, Disclosure, Event,
    HistoryRecord, InputRole, JobState, Locality, ModelId, Payload, Provider, Request, SecretRef,
    SecretString,
};
use craft_ai_native::secrets::{MemoryStore, SecretStore, standard_store};
use craft_ai_native::{ConsentFile, HistoryFile};
use craft_ai_openai::{OpenAiConfig, OpenAiProvider};
use grammarcraft_core::{AiConsentStatus, AiLocalityMode, AiMode, AiProposal, compute_word_diff};
use sha2::{Digest, Sha256};

use crate::CmdError;

const DEFAULT_CLOUD_ENDPOINT: &str = "https://api.openai.com/v1";
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

struct LocalRunnerRegistration {
    runner: Arc<dyn ModelRunner>,
    engine: String,
    model_id: ModelId,
    model_label: String,
    weights_path: PathBuf,
    expected_sha256: Option<String>,
    loaded: Option<Box<dyn LoadedModel>>,
    last_used: Option<Instant>,
}

struct HistoryMeta<'a> {
    provider_label: &'a str,
    locality: Locality,
    destination: &'a str,
    payload_bytes: usize,
}

/// Per-session AI locality, Craft [`ConsentLedger`], optional local [`ModelRunner`] / persistent files, and staged proposal.
pub struct AiSessionState {
    pub mode: AiLocalityMode,
    pub endpoint: String,
    pub consent_ledger: ConsentLedger,
    pub pending_proposal: Option<AiProposal>,
    next_proposal_id: u64,
    cloud_backend: Option<Arc<dyn AiBackend>>,
    local_registry: EngineRegistry,
    local_runner: Option<LocalRunnerRegistration>,
    consent_file: Option<ConsentFile>,
    history_file: Option<HistoryFile>,
    secret_store: Arc<dyn SecretStore>,
}

impl std::fmt::Debug for AiSessionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AiSessionState")
            .field("mode", &self.mode)
            .field("endpoint", &self.endpoint)
            .field("has_pending_proposal", &self.pending_proposal.is_some())
            .field("local_engines", &self.local_registry.engines())
            .field("has_consent_file", &self.consent_file.is_some())
            .field("has_history_file", &self.history_file.is_some())
            .field("secret_store", &self.secret_store.name())
            .finish()
    }
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
            local_registry: EngineRegistry::new(),
            local_runner: None,
            consent_file: None,
            history_file: None,
            secret_store: Arc::new(MemoryStore::default()),
        }
    }
}

impl AiSessionState {
    pub fn configure_storage(
        &mut self,
        consent_path: Option<PathBuf>,
        history_path: Option<PathBuf>,
        secret_store: Arc<dyn SecretStore>,
    ) {
        self.consent_file = consent_path.map(ConsentFile::new);
        if let Some(file) = &self.consent_file {
            // Fail closed on missing or damaged consent files.
            self.consent_ledger = file.load_or_empty();
        }
        self.history_file = history_path.map(HistoryFile::new);
        self.secret_store = secret_store;
    }

    pub fn enable_desktop_storage(&mut self) {
        if let Ok(app_id) = AppId::new("grammarcraft") {
            self.consent_file = ConsentFile::at_default_location(&app_id);
            if let Some(file) = &self.consent_file {
                self.consent_ledger = file.load_or_empty();
            }
        }
        self.history_file = HistoryFile::at_default_location();
        self.secret_store = Arc::new(standard_store(None));
    }

    pub fn store_secret(&self, key_ref: &str, value: &str) -> Result<(), CmdError> {
        let secret_ref = SecretRef::new(key_ref)
            .map_err(|e| CmdError::Params(format!("invalid secret ref `{key_ref}`: {e}")))?;
        let secret = SecretString::new(value);
        self.secret_store
            .set(&secret_ref, &secret)
            .map_err(|e| CmdError::Failed(format!("failed to store secret: {e}")))
    }

    pub fn has_secret(&self, key_ref: &str) -> Result<bool, CmdError> {
        let secret_ref = SecretRef::new(key_ref)
            .map_err(|e| CmdError::Params(format!("invalid secret ref `{key_ref}`: {e}")))?;
        self.secret_store
            .get(&secret_ref)
            .map(|opt| opt.is_some_and(|s| !s.is_empty()))
            .map_err(|e| CmdError::Failed(format!("failed to read secret: {e}")))
    }

    pub fn history_entries(&self) -> Result<Vec<HistoryEntry>, CmdError> {
        let Some(file) = &self.history_file else {
            return Ok(Vec::new());
        };
        let read = file
            .read()
            .map_err(|e| CmdError::Failed(format!("failed to read AI history: {e}")))?;
        Ok(merge(read.records, read.skipped).entries)
    }

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
        if let Some(file) = &self.consent_file {
            file.save(&self.consent_ledger)
                .map_err(|e| CmdError::Failed(format!("failed to persist AI consent: {e}")))?;
        }
        self.consent_status(0)
    }

    pub fn launcher_client_available() -> bool {
        AppId::new("grammarcraft")
            .ok()
            .and_then(|id| AiClient::standard(id).ok())
            .is_some()
    }

    pub fn register_local_runner(
        &mut self,
        runner: Arc<dyn ModelRunner>,
        weights_path: PathBuf,
        expected_sha256: Option<String>,
    ) -> Result<(), CmdError> {
        let normalized_sha = expected_sha256
            .map(|raw| normalize_sha256_hex(&raw))
            .transpose()?;
        let engine = runner.engine().trim().to_ascii_lowercase();
        if engine.is_empty() {
            return Err(CmdError::Params(
                "local model runner engine name must not be empty".to_owned(),
            ));
        }
        let model = runner.model();
        let model_label = if model.label.trim().is_empty() {
            model.id.as_str().to_owned()
        } else {
            model.label.clone()
        };
        self.local_registry.register_arc(Arc::clone(&runner));
        self.local_runner = Some(LocalRunnerRegistration {
            runner,
            engine,
            model_id: model.id,
            model_label,
            weights_path,
            expected_sha256: normalized_sha,
            loaded: None,
            last_used: None,
        });
        Ok(())
    }

    pub fn local_engines(&self) -> Vec<String> {
        self.local_registry.engines()
    }

    pub fn loaded_local_model_info(&self) -> Option<LoadedModelInfo> {
        let reg = self.local_runner.as_ref()?;
        let loaded = reg.loaded.as_ref()?;
        let model_bytes = loaded.memory_bytes();
        let estimated = model_bytes.is_none();
        let bytes = model_bytes
            .or_else(|| reg.runner.memory_needed(&reg.weights_path))
            .unwrap_or(0);
        let idle = reg.last_used.map_or(Duration::ZERO, |t| t.elapsed());
        Some(LoadedModelInfo {
            model: reg.model_id.clone(),
            label: reg.model_label.clone(),
            bytes,
            estimated,
            running: false,
            idle,
        })
    }

    pub fn unload_local_model(&mut self) -> Unloaded {
        let Some(reg) = self.local_runner.as_mut() else {
            return Unloaded::default();
        };
        if reg.loaded.take().is_some() {
            Unloaded {
                freed: vec![reg.model_id.clone()],
                after_run: Vec::new(),
            }
        } else {
            Unloaded::default()
        }
    }

    fn run_local_runner(
        &mut self,
        ai_mode: AiMode,
        original: &str,
        detail: &str,
    ) -> Result<(String, String, String), CmdError> {
        let reg = self
            .local_runner
            .as_mut()
            .ok_or_else(|| CmdError::Failed("no local model runner registered".to_owned()))?;
        if reg.loaded.is_none() {
            if let Some(expected) = &reg.expected_sha256 {
                verify_weights_sha256(&reg.weights_path, expected)?;
            }
            let loaded = reg.runner.load(&reg.weights_path).map_err(|e| {
                CmdError::Failed(format!(
                    "failed to load local model `{}`: {e}",
                    reg.model_id
                ))
            })?;
            reg.loaded = Some(loaded);
        }
        let loaded = reg
            .loaded
            .as_mut()
            .ok_or_else(|| CmdError::Failed("local model failed to initialize".to_owned()))?;
        let req = build_text_request(ai_mode, original, detail)?;
        let outputs = loaded
            .run(&req, &RunContext::detached())
            .map_err(|e| CmdError::Failed(format!("local AI execution failed: {e}")))?;
        let text = outputs
            .into_iter()
            .find_map(|out| out.payload.as_text().map(ToOwned::to_owned))
            .ok_or_else(|| {
                CmdError::Failed("local AI runner returned no text output".to_owned())
            })?;
        reg.last_used = Some(Instant::now());
        let label = format!("local-engine:{}/{}", reg.engine, reg.model_id);
        let destination = format!("local-engine://{}/{}", reg.engine, reg.model_id);
        Ok((text, label, destination))
    }

    pub fn stage_proposal(
        &mut self,
        ai_mode: AiMode,
        original: &str,
        detail: &str,
    ) -> Result<AiProposal, CmdError> {
        let (proposed, provider_label, locality, destination) = if self.local_runner.is_some() {
            let (text, label, dest) = self.run_local_runner(ai_mode, original, detail)?;
            (text, label, Locality::Local, dest)
        } else {
            self.stage_with_backend(ai_mode, original, detail)?
        };

        let hunks = compute_word_diff(original, &proposed)?;
        let proposal_id = self.next_proposal_id;
        let proposal = AiProposal {
            id: proposal_id,
            mode: ai_mode,
            provider_label: provider_label.clone(),
            original: original.to_owned(),
            proposed,
            hunks,
        };
        self.record_history(
            proposal_id,
            ai_mode,
            HistoryMeta {
                provider_label: &provider_label,
                locality,
                destination: &destination,
                payload_bytes: original.len(),
            },
        );
        self.next_proposal_id = self.next_proposal_id.saturating_add(1);
        self.pending_proposal = Some(proposal.clone());
        Ok(proposal)
    }

    fn stage_with_backend(
        &self,
        ai_mode: AiMode,
        original: &str,
        detail: &str,
    ) -> Result<(String, String, Locality, String), CmdError> {
        match self.mode {
            AiLocalityMode::LocalOffline => {
                let text = LocalOfflineBackend.generate(ai_mode, original, detail)?;
                Ok((
                    text,
                    LocalOfflineBackend.label().to_owned(),
                    Locality::Local,
                    "local://offline".to_owned(),
                ))
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
                let text = backend.generate(ai_mode, original, detail)?;
                Ok((
                    text,
                    backend.label().to_owned(),
                    Locality::Remote,
                    self.endpoint.clone(),
                ))
            }
        }
    }

    fn record_history(&self, proposal_id: u64, ai_mode: AiMode, meta: HistoryMeta<'_>) {
        let Some(file) = &self.history_file else {
            return;
        };
        let run = format!("gc-run-{proposal_id}-{}", now_ms());
        let start = HistoryRecord::Start(StartRecord {
            v: HISTORY_SCHEMA,
            run: run.clone(),
            at_ms: now_ms(),
            app: "grammarcraft".to_owned(),
            role_id: ai_mode.as_str().to_owned(),
            role_label: ai_mode.label().to_owned(),
            capability: "text.complete".to_owned(),
            connection: meta.provider_label.to_owned(),
            connection_label: meta.provider_label.to_owned(),
            model: meta.provider_label.to_owned(),
            locality: meta.locality,
            destination: meta.destination.to_owned(),
            sent: Vec::new(),
            params: vec![format!("bytes={}", meta.payload_bytes)],
            provider_job: None,
            estimate: None,
            prompt: None,
        });
        let end = HistoryRecord::End(EndRecord {
            v: HISTORY_SCHEMA,
            run,
            at_ms: now_ms(),
            outcome: Outcome::Done,
            usage: None,
        });
        let _ = file.append(&start);
        let _ = file.append(&end);
    }
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
}

fn build_text_request(ai_mode: AiMode, original: &str, detail: &str) -> Result<Request, CmdError> {
    let capability = CapabilityId::parse("text.complete")
        .map_err(|e| CmdError::Failed(format!("invalid capability id: {e}")))?;
    let instruction = system_instruction(ai_mode, detail);
    let user_prompt = if original.trim().is_empty() {
        detail.to_owned()
    } else {
        format!("{instruction}\n\n{original}")
    };
    Ok(Request::new(capability)
        .with_input(InputRole::System, Payload::text(instruction))
        .with_input(InputRole::Prompt, Payload::text(user_prompt)))
}

fn normalize_sha256_hex(raw: &str) -> Result<String, CmdError> {
    let trimmed = raw.trim();
    if trimmed.len() != 64 || !trimmed.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(CmdError::Params(
            "expected SHA-256 hex digest with 64 hexadecimal characters".to_owned(),
        ));
    }
    Ok(trimmed.to_ascii_lowercase())
}

fn verify_weights_sha256(path: &Path, expected_hex: &str) -> Result<(), CmdError> {
    let mut file = std::fs::File::open(path).map_err(|e| {
        CmdError::Failed(format!(
            "cannot open local model weights `{}`: {e}",
            path.display()
        ))
    })?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 16 * 1024];
    loop {
        let n = file.read(&mut buf).map_err(|e| {
            CmdError::Failed(format!(
                "cannot read local model weights `{}`: {e}",
                path.display()
            ))
        })?;
        if n == 0 {
            break;
        }
        if let Some(chunk) = buf.get(..n) {
            hasher.update(chunk);
        }
    }
    let digest = hasher.finalize();
    let mut actual_hex = String::with_capacity(64);
    for byte in digest {
        let _ = write!(actual_hex, "{byte:02x}");
    }
    if actual_hex != expected_hex {
        return Err(CmdError::Failed(format!(
            "local model weights SHA-256 mismatch: expected {expected_hex}, got {actual_hex}"
        )));
    }
    Ok(())
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
