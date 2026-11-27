/// Basic deterministic measurements of a document's text.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct TextStats {
    pub bytes: usize,
    pub chars: usize,
    pub words: usize,
    pub sentences: usize,
    pub paragraphs: usize,
}

impl TextStats {
    /// Measures byte length, scalar character count, words, sentences and
    /// non-empty paragraphs for `text`.
    pub fn measure(text: &str) -> Self {
        if text.is_empty() {
            return Self::default();
        }
        let words = count_words(text);
        Self {
            bytes: text.len(),
            chars: text.chars().count(),
            words,
            sentences: count_sentences(text, words),
            paragraphs: count_paragraphs(text),
        }
    }
}

fn count_words(text: &str) -> usize {
    text.split_whitespace()
        .filter(|token| token.chars().any(char::is_alphanumeric))
        .count()
}

fn count_paragraphs(text: &str) -> usize {
    text.lines().filter(|line| !line.trim().is_empty()).count()
}

fn count_sentences(text: &str, words: usize) -> usize {
    if words == 0 {
        return 0;
    }
    let mut sentences = 0usize;
    let mut saw_word_char = false;
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch.is_alphanumeric() {
            saw_word_char = true;
            continue;
        }
        if is_terminal_punct(ch) && saw_word_char {
            let followed_by_sep = chars.peek().is_none_or(|next| is_sentence_follower(*next));
            if followed_by_sep {
                sentences = sentences.saturating_add(1);
                saw_word_char = false;
            }
        }
    }
    sentences.max(1)
}

const fn is_terminal_punct(ch: char) -> bool {
    matches!(ch, '.' | '!' | '?' | '…')
}

const fn is_sentence_follower(ch: char) -> bool {
    ch.is_whitespace() || matches!(ch, '"' | '\'' | '”' | '’' | '»' | ')' | ']')
}
