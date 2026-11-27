# Feature names and trademark policy (`RFC-0029`)

This document tracks product and user-facing feature names for `GrammarCraft` under `storytold/craft-contrib` `standards/naming.md` and `rfcs/0029-other-companies-names.md`.

> **Policy note:** This is a conservative naming policy, not legal advice or formal trademark clearance. **Unclear means no:** any ambiguous name is replaced with a plain descriptive ArtCraft label or escalated to a Craft owner.

## 1. Rules summary

1. **Two-test exclusion:** Do not use a name for our feature if it is (a) another company's product, technology, or distinctive feature name, or (b) claimed as a software trademark (owner's trademark list, marked `™`/`®` in docs, or found in register searches).
2. **Scope:** Covers everything users see—app name, menus, tabs, panels, dialogs, tooltips, error/suggestion copy, READMEs, release notes, store text, and all translations. Does **not** restrict internal `command ids`, preference keys, stored values, Rust identifiers, comments, tests, or internal comparison docs.
3. **No borrowed branding or promotional comparisons:** Never use third-party icons/logos or comparative marketing copy (e.g. *"Grammarly-class"*).
4. **AI naming rule (`naming.md`):** Use `AI` (`IA` in PT-BR) for shared AI features; never expose third-party model or vendor names as feature labels.
5. **Compatibility exceptions:** File formats (`DOCX`, `Markdown`, `ODT`), open protocols (`MCP`, `LSP`), user document names, and legal notices in `ATTRIBUTION.md` / `NOTICE` stay exact.

## 2. Project name decision

| Candidate | Prefix / Status | Rationale under `RFC-0029` / `naming.md` |
| --- | --- | --- |
| **GrammarCraft** | `grammarcraft` — **Selected** | Follows ArtCraft's `<noun>Craft` suite convention (`WordCraft`, `GridCraft`, `DeckCraft`, `PDFCraft`, `PhotoCraft`). `"grammar"` is an ordinary descriptive English word (analogous to `"word"` in `WordCraft`), not an adoption of `Grammarly`. Zero visual, icon, palette, or copy resemblance to `Grammarly` is permitted. |
| **ProseCraft** | **Rejected** | Collides with an existing internal project name and external naming controversy. |
| **ProofCraft** | **Rejected** | Collides directly with `Proofcraft Pty Ltd` (`seL4` ecosystem) and sounds limited to proofreading. |
| **DraftCraft** / **EditCraft** / **StyleCraft** | **Rejected** | Less accurate functionally or higher risk of generic/CSS/CAD naming collisions. |

## 3. Proprietary names NOT used vs. ArtCraft descriptive names

> Items marked **`[Verify Rule 4]`** are ordinary descriptive terms that still require formal register searches (USPTO, EUIPO/TMview, WIPO Global Brand Database in Classes 9 and 42) and three unrelated descriptive product links before the first UI or public release PR that exposes them (`naming.md` § *Not checked yet*).

### A. Product, surfaces, and panels

| Proprietary / Competitor Name (DO NOT USE) | Reason (`RFC-0029`) | ArtCraft Descriptive Name (EN / PT-BR) | Rule 4 Status |
| --- | --- | --- | --- |
| *Grammarly*, *Grammarly Desktop*, *Grammarly Editor*, *Grammarly Pro*, *Grammarly Business* | Grammarly Inc. trademarks / product names | `GrammarCraft` / `Writing Editor` (`Editor de Escrita`) / `Team Plan` (`Plano de Equipe`) | **[Verify Rule 4]** for `GrammarCraft` |
| *GrammarlyGO*, *Grammarly AI* | Third-party AI product/brand name | `AI Assistant` (`Assistente IA`) / `AI Rewrite` (`Reescrita com IA`) | Aligned with `naming.md` (`AI`) |
| *Grammarly Assistant*, *Grammarly Sidebar*, *Grammarly Widget* | Third-party product prefix | `Review Pane` (`Painel de Revisão`) / `Floating Indicator` (`Indicador Flutuante`) | **[Verify Rule 4]** |
| *Grammarly Scorecard*, *Performance Report* | Third-party metrics panel phrasing | `Writing Summary` (`Resumo do Texto`) / `Document Metrics` (`Métricas do Documento`) | **[Verify Rule 4]** |
| *App Actions* | Competitor integration feature name | `App Integrations` (`Integrações de Aplicativos`) | **[Verify Rule 4]** |

### B. Review categories and signals

| Proprietary / Competitor Name (DO NOT USE) | Reason (`RFC-0029`) | ArtCraft Descriptive Name (EN / PT-BR) | Rule 4 Status |
| --- | --- | --- | --- |
| *Correctness* (as copied 4-pillar tab set) | Part of competitor's distinctive 4-tab taxonomy | `Spelling & Grammar` (`Ortografia e Gramática`) | **[Verify Rule 4]** (already descriptive in `wordcraft`) |
| *Clarity* (standalone as copied pillar label) | Common word used as branded pillar | `Clarity & Conciseness` (`Clareza e Concisão`) | **[Verify Rule 4]** |
| *Engagement* (standalone as copied pillar label) | Common word used as branded pillar | `Style & Vocabulary` (`Estilo e Vocabulário`) | **[Verify Rule 4]** |
| *Delivery* (standalone as copied pillar label) | Common word used as branded pillar | `Tone & Appropriateness` (`Tom e Adequação`) | **[Verify Rule 4]** |
| *Grammarly Tone Detector*, *Tone Detector* | Competitor feature mark/phrase | `Tone Analysis` (`Análise de Tom`) | **[Verify Rule 4]** |
| *Word Choice*, *Fluency* (copied subcategory labels) | Competitor UI subcategory terms | `Word Precision` (`Escolha Lexical`) / `Sentence Fluency` (`Fluência Sintática`) | **[Verify Rule 4]** |
| *Reader Reactions*, *Expert Review*, *AI Grader* | Grammarly generative feature names | `Audience Preview` (`Simulação de Leitura`) / `Criteria Review` (`Revisão por Critérios`) | **[Verify Rule 4]** |

### C. Rewrite modes and generative assistance

| Proprietary / Competitor Name (DO NOT USE) | Reason (`RFC-0029`) | ArtCraft Descriptive Name (EN / PT-BR) | Rule 4 Status |
| --- | --- | --- | --- |
| *Full-sentence rewrites*, *Strategic Clarity* | Competitor feature phrases | `Sentence Rewrite` (`Reestruturação de Frase`) / `Structural Clarity` (`Clareza Estrutural`) | **[Verify Rule 4]** |
| *Rewrite with Grammarly*, *Improve it*, *Make it assertive/persuasive* (verbatim quick-prompt copy) | Proprietary quick-action copy and mark | `Improve Clarity` (`Melhorar Clareza`), `Simplify` (`Simplificar`), `Shorten` (`Encurtar`), `Expand` (`Expandir`), `Adjust Tone` (`Ajustar Tom`), `Summarize` (`Resumir`), `Draft from Prompt` (`Gerar Rascunho`) | **[Verify Rule 4]** |

### D. Writing goals, team rules, snippets, and originality

| Proprietary / Competitor Name (DO NOT USE) | Reason (`RFC-0029`) | ArtCraft Descriptive Name (EN / PT-BR) | Rule 4 Status |
| --- | --- | --- | --- |
| *Grammarly Goals*, *Set Goals* (verbatim flow copy) | Competitor branded goal workflow | `Document Goals` (`Metas do Documento`) | **[Verify Rule 4]** |
| Copied 4-axis option strings (*Knowledgeable*, *Tell a Story* as copied quartet) | Competitor UI quartet phrasing | `Audience` (`Público-alvo`: General / Technical / Specialist), `Formality` (`Formalidade`: Informal / Neutral / Formal), `Context` (`Contexto`: General / Academic / Corporate / Technical / Creative), `Purpose` (`Objetivo`: Inform / Explain / Persuade / Narrate) | **[Verify Rule 4]** |
| *Brand Tones*, *Knowledge Share* | Competitor enterprise feature names | `Team Tone Rules` (`Regras de Tom da Equipe`), `Shared Glossary` (`Glossário Compartilhado`) | **[Verify Rule 4]** |
| *Grammarly Snippets*, *Snippets* (unchecked) | Common editor term, requires Rule 4 check | `Text Snippets` / `Reusable Snippets` (`Trechos Reutilizáveis`) | **[Verify Rule 4]** |
| *Style Guide* (unchecked) | Common term, requires Rule 4 check | `Style Rules` (`Guia de Estilo`) | **[Verify Rule 4]** |
| *Grammarly Plagiarism Checker*, *Authorship*, *Citation Generator* | Competitor feature names/marks | `Similarity Check` (`Verificação de Similaridade`), `Composition Log` (`Histórico de Composição`, if added), `Citation Formatter` (`Formatador de Citações`) | **[Verify Rule 4]** |

## 4. Pending Rule 4 verification checklist (USPTO / EUIPO / TMview / WIPO)

Before the first public UI PR or release exposing `GrammarCraft` or any `[Verify Rule 4]` label above, complete and record the checks below in the table in § 5:

- [ ] **Owner trademark lists & docs:** Search Grammarly Inc.'s published trademark list and relevant writing-assistant competitors' documentation for `™` or `®` notices.
- [ ] **USPTO search:** Search each proposed name and obvious variants on `https://tmsearch.uspto.gov/` (starting with International Classes **9** and **42**, noting registration/application status and goods/services).
- [ ] **EUIPO eSearch / TMview search:** Search each proposed name and obvious variants on `https://euipo.europa.eu/eSearch/` or `https://www.tmdn.org/tmview/`.
- [ ] **WIPO Global Brand Database search:** Search each proposed name and obvious variants on `https://branddb.wipo.int/`.
- [ ] **Three unrelated descriptive uses:** For any term justified as common descriptive phrasing, link three unrelated software products using it descriptively for their own equivalent feature.
- [ ] **Translation parity check:** Confirm all approved labels are reflected in `craft-i18n` translations (`grammarcraft.tsv`) without breaking `command ids` or stored keys.

## 5. Verification log

| Date | Name checked | Sources searched | Findings (classes 9 / 42 & status) | 3 descriptive uses | Decision |
| --- | --- | --- | --- | --- | --- |
| _Pending_ | `GrammarCraft` | Owner lists, USPTO, EUIPO/TMview, WIPO | _To be recorded before first public release / UI PR_ | N/A (`*craft` app name) | Proposed (M0) |
| _Pending_ | UI category, panel, rewrite & goal labels | Owner lists, USPTO, EUIPO/TMview, WIPO | _To be recorded before M3 UI PR_ | _To be linked_ | Proposed (`[Verify Rule 4]`) |
