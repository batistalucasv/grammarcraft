use craft_proof::ProofLang;
use craft_proof::hyphen::{Limits, hyphen_points_in};
use grammarcraft_core::{ReadabilityLevel, ReadabilityMetrics, TextStats};

use crate::clarity::word_spans;

const READING_WPM: f64 = 200.0;
const SPEAKING_WPM: f64 = 130.0;

pub(crate) fn compute_readability(
    text: &str,
    stats: &TextStats,
    lang: ProofLang,
) -> ReadabilityMetrics {
    if stats.words == 0 || stats.sentences == 0 {
        return ReadabilityMetrics::default();
    }
    let words = stats.words as f64;
    let sentences = stats.sentences as f64;
    let syllables = count_total_syllables(text, lang).max(stats.words) as f64;
    let letters = text.chars().filter(|c| c.is_alphabetic()).count() as f64;

    let words_per_sentence = words / sentences;
    let syllables_per_word = syllables / words;

    let flesch_reading_ease =
        (206.835 - 1.015 * words_per_sentence - 84.6 * syllables_per_word).clamp(0.0, 100.0);
    let flesch_kincaid_grade =
        (0.39 * words_per_sentence + 11.8 * syllables_per_word - 15.59).max(0.0);
    let gulpease_index = (89.0 + (300.0 * sentences - 10.0 * letters) / words).clamp(0.0, 100.0);

    let level = classify_level(flesch_reading_ease);
    ReadabilityMetrics {
        flesch_reading_ease,
        flesch_kincaid_grade,
        gulpease_index,
        reading_minutes: words / READING_WPM,
        speaking_minutes: words / SPEAKING_WPM,
        level,
    }
}

fn count_total_syllables(text: &str, lang: ProofLang) -> usize {
    let limits = Limits::default();
    word_spans(text)
        .into_iter()
        .map(|(_, _, word)| {
            let hyphen_syllables = hyphen_points_in(word, &limits, lang)
                .len()
                .saturating_add(1);
            let vowel_syllables = count_vowel_groups(word);
            hyphen_syllables.max(vowel_syllables).max(1)
        })
        .sum()
}

fn count_vowel_groups(word: &str) -> usize {
    let mut groups = 0usize;
    let mut in_vowel = false;
    for ch in word.chars() {
        let vowel = is_vowel(ch);
        if vowel && !in_vowel {
            groups = groups.saturating_add(1);
        }
        in_vowel = vowel;
    }
    groups.max(1)
}

const fn is_vowel(ch: char) -> bool {
    matches!(
        ch,
        'a' | 'e'
            | 'i'
            | 'o'
            | 'u'
            | 'y'
            | 'A'
            | 'E'
            | 'I'
            | 'O'
            | 'U'
            | 'Y'
            | 'á'
            | 'à'
            | 'â'
            | 'ã'
            | 'é'
            | 'ê'
            | 'í'
            | 'ó'
            | 'ô'
            | 'õ'
            | 'ú'
            | 'ü'
            | 'Á'
            | 'À'
            | 'Â'
            | 'Ã'
            | 'É'
            | 'Ê'
            | 'Í'
            | 'Ó'
            | 'Ô'
            | 'Õ'
            | 'Ú'
            | 'Ü'
    )
}

fn classify_level(flesch: f64) -> ReadabilityLevel {
    if flesch >= 80.0 {
        ReadabilityLevel::VeryEasy
    } else if flesch >= 65.0 {
        ReadabilityLevel::Easy
    } else if flesch >= 50.0 {
        ReadabilityLevel::Standard
    } else if flesch >= 30.0 {
        ReadabilityLevel::Difficult
    } else {
        ReadabilityLevel::Academic
    }
}
