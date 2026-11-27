use crate::{CoreError, CoreResult};

/// Maximum accepted byte length for a goal option string.
pub const MAX_GOAL_VALUE_BYTES: usize = 64;

/// Target audience axis for writing rules and readability weighting.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Audience {
    #[default]
    General,
    Specialized,
    TechnicalExecutive,
}

impl Audience {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::General => "general",
            Self::Specialized => "specialized",
            Self::TechnicalExecutive => "technical_executive",
        }
    }

    pub fn parse(raw: &str) -> CoreResult<Self> {
        let normalized = normalize_goal_token("audience", raw)?;
        match normalized.as_str() {
            "general" => Ok(Self::General),
            "specialized" => Ok(Self::Specialized),
            "technical_executive" | "technical-executive" | "executive" => {
                Ok(Self::TechnicalExecutive)
            }
            _ => Err(CoreError::InvalidGoal {
                field: "audience",
                value: normalized,
            }),
        }
    }
}

/// Target formality axis.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Formality {
    Informal,
    #[default]
    Neutral,
    Formal,
}

impl Formality {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Informal => "informal",
            Self::Neutral => "neutral",
            Self::Formal => "formal",
        }
    }

    pub fn parse(raw: &str) -> CoreResult<Self> {
        let normalized = normalize_goal_token("formality", raw)?;
        match normalized.as_str() {
            "informal" => Ok(Self::Informal),
            "neutral" => Ok(Self::Neutral),
            "formal" => Ok(Self::Formal),
            _ => Err(CoreError::InvalidGoal {
                field: "formality",
                value: normalized,
            }),
        }
    }
}

/// Document context / domain axis.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Context {
    #[default]
    General,
    Academic,
    Corporate,
    Technical,
    Creative,
    Casual,
}

impl Context {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::General => "general",
            Self::Academic => "academic",
            Self::Corporate => "corporate",
            Self::Technical => "technical",
            Self::Creative => "creative",
            Self::Casual => "casual",
        }
    }

    pub fn parse(raw: &str) -> CoreResult<Self> {
        let normalized = normalize_goal_token("context", raw)?;
        match normalized.as_str() {
            "general" => Ok(Self::General),
            "academic" => Ok(Self::Academic),
            "corporate" | "business" => Ok(Self::Corporate),
            "technical" => Ok(Self::Technical),
            "creative" => Ok(Self::Creative),
            "casual" => Ok(Self::Casual),
            _ => Err(CoreError::InvalidGoal {
                field: "context",
                value: normalized,
            }),
        }
    }
}

/// Communicative purpose axis.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Purpose {
    #[default]
    Inform,
    Explain,
    Persuade,
    Narrate,
}

impl Purpose {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Inform => "inform",
            Self::Explain => "explain",
            Self::Persuade => "persuade",
            Self::Narrate => "narrate",
        }
    }

    pub fn parse(raw: &str) -> CoreResult<Self> {
        let normalized = normalize_goal_token("purpose", raw)?;
        match normalized.as_str() {
            "inform" => Ok(Self::Inform),
            "explain" | "describe" => Ok(Self::Explain),
            "persuade" => Ok(Self::Persuade),
            "narrate" => Ok(Self::Narrate),
            _ => Err(CoreError::InvalidGoal {
                field: "purpose",
                value: normalized,
            }),
        }
    }
}

/// Four-axis writing goal configuration for a document session.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WritingGoals {
    pub audience: Audience,
    pub formality: Formality,
    pub context: Context,
    pub purpose: Purpose,
}

fn normalize_goal_token(field: &'static str, raw: &str) -> CoreResult<String> {
    if raw.len() > MAX_GOAL_VALUE_BYTES {
        return Err(CoreError::InvalidGoal {
            field,
            value: "<oversized>".to_owned(),
        });
    }
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(CoreError::InvalidGoal {
            field,
            value: String::new(),
        });
    }
    Ok(trimmed.to_ascii_lowercase())
}
