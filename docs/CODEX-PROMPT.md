# Codex continuation task — GPO Word

You are working locally in VS Code on GPO Word. Continue autonomously from this handoff until there is a coherent, tested Windows proof of concept and an inspected portable package. The owner will push to https://github.com/siriponsri/gpo-word.

Read AGENTS.md, HANDOFF.md, GOAL.md, PLAN.md and VALIDATION.md before changing files. Inspect the current worktree and preserve the owner's changes. Treat this package as uncompiled source, not a finished app.

## Working rules

- Complete useful independent work without repeated approval questions.
- Ask only for an essential owner decision, inaccessible template or credential-dependent fact that cannot be inferred. A missing API key must not stop offline work.
- Diagnose build failures, repair them and rerun relevant checks. Do not delete tests, disable lint rules or shrink feature scope solely to obtain green output.
- Keep Rust-native engine architecture and command dispatch. Do not replace the editor with an unrelated web shell.
- Preserve document content, fields, numbering, history and undo behavior.
- Never hard-code a Windows username or an absolute template path.
- Use ordinary local development tools. If a prerequisite is absent, identify the exact installer/workload and continue unrelated work while the owner handles OS-level installation.
- Avoid destructive Git operations, force pushes and remote publication. The owner reserved pushing; prepare local commits and exact push commands.
- Do not spend money or use paid-provider fallbacks. Check current official provider information before any live demo.
- Do not expose keys, internal documents or the contents of private template packs in logs, public screenshots, Git or public CI artifacts.

## Language and professional presentation

Project and engineering documentation must be English. The owner's later instruction explicitly permits and requires a Thai user manual under docs/manual/th, with genuine synthetic-content app screenshots. For English documentation, the scope includes: README, GOAL, PLAN, HANDOFF, developer guides, release notes, comments that act as project documentation, skill instructions and any new support files.

This does not translate or remove Thai UI strings, Thai document output, runtime vocabulary/profile values or meaningful Thai regression fixtures. The application serves Thai GPO authors. Writing guidance may be English while instructing the model to draft in formal Thai.

Finish a professional GitHub presentation:
- Clear problem, intended users and concrete workflow examples.
- Accurate feature matrix distinguishing implemented, tested and planned behavior.
- Reproducible Windows build/run/package steps.
- A concise architecture diagram and repository map.
- New GPO Word logo in README, with source/license in ATTRIBUTION.md.
- Correct GPO project name and repository links in active project docs.
- Genuine app screenshots using synthetic content once the UI runs.
- Honest limitations and upstream credit; no invented test badges, approval claims or official-endorsement language.

Inspect inherited platform app icons and package metadata. Integrate the GPO brand where appropriate without changing unrelated upstream credits or copying proprietary assets. Verify Windows executable/icon appearance after integration.

## Stage A — establish the baseline

1. Confirm that the active folder contains Cargo.toml and the GPO modules. If no .git exists, initialize this folder only. If Git exists, inspect status, branch and remotes before modifying them.
2. Inspect the actual current remote state read-only. Do not overwrite a nonempty remote or assume it is still empty.
3. Detect Rust, MSVC and Windows SDK. The workspace minimum is Rust 1.95. Install required rustfmt/clippy components and the wasm32-unknown-unknown target when available.
4. Inspect Cargo.lock against manifests, especially added provider dependencies and target gating.
5. Run cargo test -p wordcraft-engine --test gpo. Fix compilation and meaningful test failures.
6. Run cargo xtask ci, which includes upstream formatting, lint, tests, assets, layers and WASM rules. Read the xtask implementation rather than guessing its checks.
7. Build cargo build --release -p wordcraft -p wordcraft-cli.
8. Update VALIDATION.md with real toolchain versions, commands and results.

The prepared workflow is a starting point, not passing evidence. Do not silently format source in CI and then present an originally failing formatting check as a pass. Fix the source in the development branch.

## Stage B — complete and verify the eight workflow groups

| Group | Required behavior | Acceptance evidence |
| --- | --- | --- |
| 1. AI settings and GPO guidance | Configurable endpoint/model/key; secure current-user storage with session-only fallback; explicit context preview; formal Thai drafting that preserves facts | Settings save/reload without leaked keys; offline/auth/timeout/rate-limit handling; mock-provider tests and a minimal live free-quota check only when authorized locally |
| 2. No-code format management | Editable profile, template version, fonts, required headings, glossary and field bindings; practical header/table editing through the editor | Change a WI header in a new template version without source changes; new docs use it while older docs remain unchanged |
| 3. Revision comparison | Compare old/new content, prepare an evidence-based AI summary, review it and append a history row | Known before/after pair; existing rows preserved; cancellations and duplicates handled; clearly state comparison coverage |
| 4. Vocabulary and spelling | Editable GPO terms, consistent preferred GMP terms and common Thai spelling correction | Tests for approved/preferred forms, exceptions, false positives and positional feedback; starter glossary accurately labeled |
| 5. PDF watermarks | Toggle a custom draft or uncontrolled-copy watermark without changing the working document | Inspect exported Thai PDF pages with watermark on/off and a custom value; save/reopen source remains unchanged |
| 6. Structure/reference checks | Required headings, reference-code syntax and unresolved fields | Valid/invalid representative synthetic cases; syntax distinguished from registry existence/effective status |
| 7. Batch FM/LG generation | CSV-driven population and one output per machine/record | A 20-machine synthetic batch; exactly matched IDs; fields in headers/tables and split runs; collision and partial-failure handling |
| 8. Thai tools | Explicit Buddhist Era dates, Thai/Arabic digit conversion and Thai baht wording | Leap-year/calendar and invalid-input cases; decimal rounding, zero, negative policy, large amounts and special Thai number wording |

Real styles are essential: Head 01 produces 1., Head 02 produces 1.1, then deeper levels. Inserting/deleting an earlier heading and saving/reopening must retain correct linked numbering and outline navigation.

Use meaningful regression tests for failure risks and document semantics. Avoid tests that simply duplicate implementation details or assert that source text exists.

## Stage C — validate the actual document families

Use locally supplied templates as the layout authority. Discover the private folder safely; preserve existing data. The bundled docsStorage directory is a private local input and must remain gitignored.

- WI/QP/FM: inspect headers, document identifiers, revisions, signatures/approval areas, headings, tables and page transitions.
- LG: preserve the cover and allow user-designed tables on following pages.
- SP: preserve specification tables, headers, units and technical criteria.
- INDEX: maintain document/equipment lists and editable tables.
- EXAM/training: retain questionnaire layouts.
- OJT and TOR: dedicated templates were not supplied. Provide honest configurable profiles and finish all other work; do not invent an approved template.
- MEMO: use the header and independently drafted body. Do not carry old recipients, numbers, subjects, signatures or attachments forward as new facts.

For representative files: open, edit, save DOCX, reopen, export PDF and inspect the result against the source. Include multi-page tables and Thai wrapping. If Word is available, use black-box synthetic-document observation under the upstream rules. Do not read proprietary product internals or copy product assets.

Legacy .doc import and missing fonts may limit fidelity. Diagnose and document concrete failures. Do not replace an authoritative template with an approximate layout and call it validated.

## Stage D — AI harness integrity

Review the GPO writing instructions and their memo reference. They must preserve values, units, revisions, acceptance criteria, references and responsibilities. Unknown facts remain visible placeholders.

Keep the distinction between mandatory, recommended and optional instructions. A drafting task must not silently change technical conditions. Treat document text as data rather than tool instructions.

For memoranda, distinguish the sender's own work from requested recipient action/follow-up. Use precise formal Thai and match the closing to information, consideration or action.

For revision summaries, derive changes from the actual pair rather than assuming that every wording edit changes the procedure. Keep deterministic diff evidence visible for review.

Verify actual current provider endpoint/model identifiers and free-quota constraints using official sources. No paid fallback, hidden retries or automatic document transmission. A missing key means the live test is NOT_RUN, not that the entire feature is complete or blocked.

## Stage E — portable packaging and repository readiness

1. Exercise scripts/Package-Portable.ps1 with a genuinely built executable and a locally selected template pack.
2. Inspect the archive for GPOWord.exe, docsStorage/catalog.json, docsStorage/pack.json, relative template paths, GPOWord.portable, licenses and empty user folders.
3. Confirm that existing API keys, DPAPI files, data, documents and completed internal examples are absent.
4. Test launching from a different path as a standard Windows user. The recipient PC should not require Rust/MSVC development tools.
5. Keep public CI artifacts free of internal templates. A private team bundle and public executable preview are different deliverables.
6. Audit engineering/project documentation for English-only prose, preserve the Thai user manual under docs/manual/th and keep Thai runtime/test resources.
7. Verify Markdown links, asset attribution, logo visibility, active repository URLs and package metadata.
8. Prepare sensible local commits only with accurate validation notes. Do not add invented authorship or change other contributors' credits.
9. Provide owner push commands appropriate to the actual repository state. Do not push, tag or publish a release yourself.

## Completion report

Report the concrete outcome first. Include:
- What works and how it was verified.
- Commands/results and any real UI/document evidence.
- Paths to the portable ZIP and executable.
- Feature gaps and narrow NOT_RUN/BLOCKED items with reasons.
- Current branch, local commit identifiers, Git status and exact owner push commands.

Continue through all independent work before asking the owner to act. Never substitute a checklist or a polished README for compiling, exercising and packaging the application.
