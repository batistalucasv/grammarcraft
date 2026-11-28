# Clean-Room Derivation — Deep Grammar & Clarity Rules

> **Scope:** Deterministic linguistic rules in `crates/rules/src/grammar_deep.rs`, `crates/rules/src/clarity.rs`, and `crates/rules/src/scoring.rs`.  
> **Clean-room status:** Authored exclusively from public grammatical and editorial conventions for English (`en`), Portuguese (`pt`), and Spanish (`es`), without inspecting any proprietary rule database or copyleft (`GPL`/`LGPL`) grammar engine.

## 1. Deep grammar rules (`crates/rules/src/grammar_deep.rs`)

All rules run per paragraph on UTF-8-safe byte spans (`para_text.to_lowercase()` guarded by `lower.len() == para_text.len()` and word-boundary checks via `is_word_boundary`) and are strictly isolated by `ProofLang`:

| Rule ID | Language | Category / Severity | Linguistic basis & deterministic pattern |
| --- | --- | --- | --- |
| `grammar.comma_splice` | `en*` | `SpellingAndGrammar` / `Warning` | Detects `, <conjunctive adverb>, ` (`however`, `therefore`, `moreover`, `furthermore`) bounded by alphanumeric/alphabetic characters; suggests `; <adverb>, ` or `. <Adverb>, `. |
| `grammar.subject_verb_agreement` | `en*` | `SpellingAndGrammar` / `Warning` | Detects singular head-noun subjects (`the list of ...`, `each of the ...`, `neither of ...`, `the number of ...`) paired with plural copula `are` / `were`, suggesting `is` / `was`. |
| `grammar.less_vs_fewer` | `en*` | `SpellingAndGrammar` / `Suggestion` | Detects `less <countable plural>` against an explicit allowlist (`EN_COUNTABLE_PLURALS`, e.g., `apples`, `bugs`, `errors`, `files`, `issues`, `users`, `words`), suggesting `fewer <noun>`. |
| `grammar.pt_crase` | `pt*` | `SpellingAndGrammar` / `Warning` | Conservative multi-word locution checks (`PT_CRASE_RULES`): missing grave accent (`a medida que` → `à medida que`, `a proporção que` → `à proporção que`), forbidden crasis before infinitive (`à partir` → `a partir`), repeated-word expressions (`dia à dia`, `frente à frente`, `lado à lado`, `cara à cara`, `gota à gota`), and standard register (`a nível de` → `em nível de`). |
| `grammar.pt_existential` | `pt*` | `SpellingAndGrammar` / `Warning` | Impersonal existential `houveram` → `houve`, and impersonal elapsed-time `fazem <num|count word> <time unit>` (`PT_TIME_UNITS`, `PT_COUNT_WORDS`) → `faz <...>`. |
| `grammar.es_dequeismo` | `es*` | `SpellingAndGrammar` / `Warning` | Detects `<verb> de que` for declarative verbs (`ES_DEQUEISMO_VERBS`: `pienso`, `creo`, `opino`, `considero`, `digo`), suggesting `<verb> que`. |
| `grammar.es_existential` | `es*` | `SpellingAndGrammar` / `Warning` | Detects pluralized existential `hubieron` (outside compound-tense contexts handled conservatively), suggesting impersonal `hubo`. |

## 2. Clarity & syntactic rhythm rules (`crates/rules/src/clarity.rs` & `scoring.rs`)

| Rule ID | Language | Category / Severity | Linguistic basis & goal modulation |
| --- | --- | --- | --- |
| `clarity.nominalization` | `en*`, `pt*` | `Clarity` / `Suggestion` | Flags heavy verb-plus-noun nominalizations (`make a decision` → `decide`, `conduct an investigation` → `investigate`, `give consideration to` → `consider`, `take into consideration` → `consider`; PT-BR `tomar uma decisão` → `decidir`, `fazer uma análise` → `analisar`, `realizar uma verificação` → `verificar`) preserving initial casing (`match_initial_case`). |
| `clarity.monotonous_rhythm` | `en*`, `pt*`, `es*` | `Clarity` / `Suggestion` | Uses `scoring::has_uniform_sentence_rhythm(&word_counts, min_run)` to detect runs of sentences (each `>= 5` words) whose lengths stay within a `<= 2`-word spread. Default `min_run = 3`; relaxed to `min_run = 4` when `WritingGoals` targets `Audience::Specialized` and `Purpose::Inform` (`Technical` context). |
