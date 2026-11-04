# GPO Word — development handoff

## Objective

Continue validating this Windows proof of concept and portable preview for teammates who cannot install development tools. Maintain professional English repository documentation and the project's independent brand.

Canonical repository: https://github.com/siriponsri/gpo-word

The owner intends to push the result. Prepare local changes, commits and push instructions; do not push, create remote releases or publish private data automatically.

## First read

Read AGENTS.md, GOAL.md, PLAN.md, VALIDATION.md and docs/CODEX-PROMPT.md. The last file contains the full continuation task and feature acceptance matrix.

The root HANDOFF.md is now GPO-specific. The former KhianKrub handoff is retained as docs/upstream/khiankrub-handoff.md for historical context. Upstream roadmap completion figures do not prove that GPO additions pass.

## Source and provenance

This local all-in-one package includes application source, a Windows preview executable, default profiles, scripts, English documentation, a README logo and private templates under gitignored docsStorage. It contains no Git history. Keep the private templates out of the public repository.

The originating local source commit was 3ef3172; the archive manifest records the complete source commit identifier. The upstream references are pinned in gpo/upstreams.lock.json. License and attribution files are retained.

If this extracted directory has no .git, initialize it locally. Inspect any existing repository and origin before changing it. Never reset, clean or force-push the owner's work. An empty canonical remote was observed while preparing this handoff; inspect it again before proposing push commands.

## Where to work

Open the inner gpo-word directory in VS Code: it contains Cargo.toml, crates, apps, scripts and this file. Do not open an upstream snapshot as the active project and do not initialize Git in a parent folder containing private resources.

The public repository excludes docsStorage, private, data, documents, target, dist, credentials and runtime settings.

The private template pack is already present under docsStorage. Preserve this folder and the owner's template changes. If templates are outside the project, configure their location instead of copying all of them blindly.

## Known state

The Windows GNU development build and selected synthetic workflows have run. Full CI stopped at Clippy and live provider calls and private-template fidelity are not established. See VALIDATION.md for the exact evidence.

Potential build issues include dependency-lock drift, trait/API mismatches, platform-specific key storage and WASM separation. Diagnose from actual compiler output rather than assuming a particular failure exists.

## Key modules

- crates/engine/src/gpo.rs: profiles, styles, deterministic checks/utilities, comparison, template fields and draft context.
- crates/engine/tests/gpo.rs: proposed GPO integration tests; run and assess their coverage.
- crates/engine/src/cmd/gpo.rs: registered checks, fill, comparison, revision, export and batch commands.
- apps/wordcraft/src/gpo.rs: workbench, no-code controls, optional AI calls and DPAPI integration.
- apps/wordcraft/src/gpo_storage.rs: template discovery, relative paths and portable data.
- scripts/Start-GPOWord.ps1: source build/launch helper.
- scripts/Package-Portable.ps1: local executable-plus-template ZIP.
- .github/workflows/gpo-windows.yml: public Windows checks and source-only preview build.

## Template facts

The prepared private pack has 16 entries: 15 source templates copied unchanged, plus one application-native memorandum-header preset. It covers WI/QP/FM, logbook covers, SP, index/equipment lists and training questionnaires.

No dedicated OJT or TOR template was supplied. Do not select an examination template as OJT automatically.

Some authoritative templates are legacy .doc. Validate parser behavior and layout explicitly. Installed fonts can affect appearance; do not bundle proprietary fonts.

Logbook work centers on the cover. Users design following tables. Memoranda use the template header while bodies are drafted independently; examples must not leak old recipients, subject lines or signatures.

## Important implementation boundaries

- Real numbering must remain linked to paragraph styles.
- The existing comparison is not a full visual/direct-formatting diff.
- Reference checking validates syntax and placeholders, not authoritative registry status.
- The starter glossary contains a few common corrections, not an approved GMP dictionary.
- AI-generated summaries require review before inserting history.
- Header changes require editing and saving a template, not merely changing a JSON profile.
- DPAPI is current-user storage. Never package existing keys or ask the owner to paste one into the chat or source.
- Provider defaults are editable. Verify current model names/free quotas before a live call; use no paid fallback.

## Documentation and branding

Project and engineering documentation is English. The owner explicitly requested a Thai user manual with genuine app screenshots under docs/manual/th. Thai runtime UI, document output, dictionaries/profiles and test data remain supported. English writing guidance explicitly requests Thai drafts where appropriate.

The new logo is in assets/brand/gpo-word-logo.png and appears in README.md. Its ICO is embedded in the Windows build. Keep all attribution and notices.

Use genuine synthetic-content app captures after the build works. Do not label an illustration or mockup as a running-app screenshot.

## Verification and final delivery

Run targeted tests, cargo xtask ci and the release build. Exercise all eight requested workflow groups. Keep a compact evidence log with actual PASS/FAIL/NOT_RUN statuses.

Create a public preview without internal templates and a private portable package only with locally selected templates. Test the latter as a standard user from a different path.

Leave English documentation, a clean/reviewable diff, local commits where checks permit, a tested ZIP when possible and precise owner push commands. If a required owner input is missing, complete all independent work and name the narrow remaining blocker.
