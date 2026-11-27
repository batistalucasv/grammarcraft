use std::collections::BTreeMap;

use grammarcraft_core::{TextSpan, ToneProfile, ToneSignal, ToneSignalKind};

use crate::clarity::word_spans;

const MAX_DOMINANT_TONES: usize = 3;
const MAX_EVIDENCE_SPANS: usize = 8;

const CUE_WORDS: &[(&str, ToneSignalKind)] = &[
    ("maybe", ToneSignalKind::Hesitant),
    ("perhaps", ToneSignalKind::Hesitant),
    ("possibly", ToneSignalKind::Hesitant),
    ("talvez", ToneSignalKind::Hesitant),
    ("aparentemente", ToneSignalKind::Hesitant),
    ("urgent", ToneSignalKind::Urgent),
    ("immediately", ToneSignalKind::Urgent),
    ("asap", ToneSignalKind::Urgent),
    ("urgente", ToneSignalKind::Urgent),
    ("imediatamente", ToneSignalKind::Urgent),
    ("therefore", ToneSignalKind::Formal),
    ("furthermore", ToneSignalKind::Formal),
    ("regarding", ToneSignalKind::Formal),
    ("portanto", ToneSignalKind::Formal),
    ("conforme", ToneSignalKind::Formal),
    ("prezado", ToneSignalKind::Formal),
    ("please", ToneSignalKind::Cordial),
    ("thank", ToneSignalKind::Cordial),
    ("thanks", ToneSignalKind::Cordial),
    ("appreciate", ToneSignalKind::Cordial),
    ("obrigado", ToneSignalKind::Cordial),
    ("obrigada", ToneSignalKind::Cordial),
    ("agradeço", ToneSignalKind::Cordial),
    ("must", ToneSignalKind::Assertive),
    ("clearly", ToneSignalKind::Assertive),
    ("guarantee", ToneSignalKind::Assertive),
    ("devemos", ToneSignalKind::Assertive),
    ("claramente", ToneSignalKind::Assertive),
    ("garantimos", ToneSignalKind::Assertive),
    ("however", ToneSignalKind::Analytical),
    ("consequently", ToneSignalKind::Analytical),
    ("evidence", ToneSignalKind::Analytical),
    ("contudo", ToneSignalKind::Analytical),
    ("dados", ToneSignalKind::Analytical),
    ("evidência", ToneSignalKind::Analytical),
    ("understand", ToneSignalKind::Empathetic),
    ("compreendo", ToneSignalKind::Empathetic),
    ("failed", ToneSignalKind::Critical),
    ("broken", ToneSignalKind::Critical),
    ("falhou", ToneSignalKind::Critical),
];

pub(crate) fn analyze_tone(text: &str) -> ToneProfile {
    if text.trim().is_empty() {
        return ToneProfile::default();
    }
    let mut buckets: BTreeMap<ToneSignalKind, Vec<TextSpan>> = BTreeMap::new();
    for (start, end, word) in word_spans(text) {
        let lower = word.to_lowercase();
        if let Some((_, kind)) = CUE_WORDS.iter().find(|(cue, _)| *cue == lower)
            && let Ok(span) = TextSpan::new(text, start, end)
        {
            push_span(&mut buckets, *kind, span);
        }
    }
    for (idx, ch) in text.char_indices() {
        if ch == '!'
            && let Ok(span) = TextSpan::new(text, idx, idx + ch.len_utf8())
        {
            push_span(&mut buckets, ToneSignalKind::Urgent, span);
        }
    }

    let total_hits: usize = buckets.values().map(Vec::len).sum();
    if total_hits == 0 {
        return ToneProfile::default();
    }

    let mut signals: Vec<ToneSignal> = buckets
        .into_iter()
        .map(|(kind, evidence_spans)| {
            let share = (evidence_spans.len() * 100) / total_hits;
            let strength = u8::try_from(share.clamp(1, 100)).unwrap_or(100);
            ToneSignal {
                kind,
                strength,
                evidence_spans,
            }
        })
        .collect();
    signals.sort_by(|a, b| {
        b.strength
            .cmp(&a.strength)
            .then_with(|| a.kind.cmp(&b.kind))
    });
    signals.truncate(MAX_DOMINANT_TONES);

    ToneProfile { dominant: signals }
}

fn push_span(
    buckets: &mut BTreeMap<ToneSignalKind, Vec<TextSpan>>,
    kind: ToneSignalKind,
    span: TextSpan,
) {
    let entry = buckets.entry(kind).or_default();
    if entry.len() < MAX_EVIDENCE_SPANS {
        entry.push(span);
    }
}
