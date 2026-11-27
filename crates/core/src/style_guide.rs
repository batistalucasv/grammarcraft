use serde::{Deserialize, Serialize};

use crate::{CoreError, CoreResult};

/// Maximum declarative style rules held in a [`StyleGuide`].
pub const MAX_STYLE_RULES: usize = 2_000;

/// Maximum UTF-8 byte length for a rule id, term, pattern, or rationale field.
pub const MAX_RULE_PATTERN_BYTES: usize = 256;

/// Kind of declarative team/editorial rule in a [`StyleGuide`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StyleRuleKind {
    PreferredTerm {
        discouraged: String,
        preferred: String,
        rationale: String,
    },
    BrandCasing {
        exact: String,
        rationale: String,
    },
    PatternRule {
        pattern: String,
        replacement: Option<String>,
        rationale: String,
    },
}

/// One validated style rule identified by a unique `id`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StyleRule {
    pub id: String,
    pub kind: StyleRuleKind,
}

impl StyleRule {
    pub fn new(id: &str, kind: StyleRuleKind) -> CoreResult<Self> {
        let id = bounded_non_empty("id", id)?.to_owned();
        let kind = validate_kind(kind)?;
        Ok(Self { id, kind })
    }
}

/// Bounded collection of team terminology, brand casing, and pattern rules.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StyleGuide {
    rules: Vec<StyleRule>,
}

impl StyleGuide {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a rule if `id` is not already registered. Returns `Ok(false)` on duplicate id.
    pub fn add_rule(&mut self, rule: StyleRule) -> CoreResult<bool> {
        let validated = StyleRule::new(&rule.id, rule.kind)?;
        if self
            .rules
            .iter()
            .any(|existing| existing.id == validated.id)
        {
            return Ok(false);
        }
        if self.rules.len() >= MAX_STYLE_RULES {
            return Err(CoreError::TooManyStyleRules {
                max: MAX_STYLE_RULES,
            });
        }
        self.rules.push(validated);
        Ok(true)
    }

    pub fn remove_rule(&mut self, id: &str) -> CoreResult<bool> {
        let id = bounded_non_empty("id", id)?;
        let Some(pos) = self.rules.iter().position(|r| r.id == id) else {
            return Ok(false);
        };
        self.rules.remove(pos);
        Ok(true)
    }

    pub fn rules(&self) -> &[StyleRule] {
        &self.rules
    }

    pub fn len(&self) -> usize {
        self.rules.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }
}

fn bounded_non_empty<'a>(field: &'static str, raw: &'a str) -> CoreResult<&'a str> {
    if raw.len() > MAX_RULE_PATTERN_BYTES {
        return Err(CoreError::RulePatternTooLong {
            max: MAX_RULE_PATTERN_BYTES,
        });
    }
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(CoreError::InvalidStyleRule(format!(
            "`{field}` must not be empty"
        )));
    }
    Ok(trimmed)
}

fn validate_kind(kind: StyleRuleKind) -> CoreResult<StyleRuleKind> {
    match kind {
        StyleRuleKind::PreferredTerm {
            discouraged,
            preferred,
            rationale,
        } => Ok(StyleRuleKind::PreferredTerm {
            discouraged: bounded_non_empty("discouraged", &discouraged)?.to_owned(),
            preferred: bounded_non_empty("preferred", &preferred)?.to_owned(),
            rationale: bounded_non_empty("rationale", &rationale)?.to_owned(),
        }),
        StyleRuleKind::BrandCasing { exact, rationale } => Ok(StyleRuleKind::BrandCasing {
            exact: bounded_non_empty("exact", &exact)?.to_owned(),
            rationale: bounded_non_empty("rationale", &rationale)?.to_owned(),
        }),
        StyleRuleKind::PatternRule {
            pattern,
            replacement,
            rationale,
        } => {
            let pattern = bounded_non_empty("pattern", &pattern)?.to_owned();
            let replacement = match replacement {
                Some(rep) => {
                    if rep.len() > MAX_RULE_PATTERN_BYTES {
                        return Err(CoreError::RulePatternTooLong {
                            max: MAX_RULE_PATTERN_BYTES,
                        });
                    }
                    Some(rep)
                }
                None => None,
            };
            let rationale = bounded_non_empty("rationale", &rationale)?.to_owned();
            Ok(StyleRuleKind::PatternRule {
                pattern,
                replacement,
                rationale,
            })
        }
    }
}
