# Delivery plan

This plan is driven by exit evidence. Continue independent work through routine implementation choices; ask the owner only for information that cannot be obtained from the repository or local templates.

## Starting point

The source is based on the pinned KhianKrub snapshot, with WordCraft as a second upstream. GPO engine commands, the workbench, portable storage and private template catalog support are implemented in source. No successful GPO Rust build, desktop smoke test or live API check has been established.

The home-PC agent must inspect current code and rerun checks rather than treating this handoff as proof of functionality.

## Outcome 1 — a reproducible Windows build

- Inspect AGENTS.md, HANDOFF.md, Cargo manifests, command registration and GPO modules.
- Detect installed Rust, MSVC and Windows SDK before proposing installations.
- Resolve compilation, formatting, lint, dependency and platform-gating problems at their cause.
- Preserve the Rust-native architecture, command registry and WASM-safe lower layers.
- Refresh Cargo.lock if required and review dependency changes.
- Run GPO tests and cargo xtask ci; build the desktop and CLI release binaries.
- Record commands, versions and actual results in VALIDATION.md.

Exit evidence: a repeatable clean build and meaningful passing checks. Do not weaken checks to manufacture a green result.

## Outcome 2 — trustworthy document workflows

- Verify actual linked heading styles and outline behavior, including save/reopen.
- Open representative private WI/QP/FM/SP/LG/index/training/memo templates.
- Inspect headers, page geometry, tables, fonts and pagination against the source.
- Keep old DOC handling explicit; diagnose parser/conversion limitations rather than promising DOCX fidelity.
- Verify template field binding across runs and headers.
- Support routine no-code profile and layout updates without silently changing existing documents.

Exit evidence: saved DOCX and PDF examples with a documented comparison to their input templates. Keep these examples local if they contain organization data.

## Outcome 3 — useful review and document-control support

- Verify required headings, reference syntax, placeholders and terminology exceptions.
- Review change detection against a known before/after pair.
- Keep text/style comparison distinct from visual and full direct-formatting comparison.
- Append reviewed revision-history rows without altering earlier entries.
- Create a 20-record synthetic batch and verify content, file names, collisions and partial failures.
- Exercise Thai calendar, digit and amount utilities at meaningful boundaries.

Exit evidence: reproducible synthetic cases and representative template smoke tests.

## Outcome 4 — optional, controlled AI assistance

- Keep endpoint/model configurable and all calls explicitly initiated.
- Check current provider model availability and quota information from official documentation.
- Verify key save/load in a normal Windows account and session-only fallback if DPAPI fails.
- Exercise offline, invalid key, timeout and rate-limit paths.
- Perform a minimal live demo only if a locally provided key and current free quota permit it.
- Preserve technical facts and exact distinctions between mandatory, recommended and optional wording.

Exit evidence: deterministic tests plus an explicitly labeled live result or NOT_RUN reason. No paid fallback, hidden retry loop or automatic document upload.

## Outcome 5 — professional, portable handoff

- Keep project and engineering documentation in English, with the owner-requested Thai user manual under docs/manual/th.
- Preserve Thai UI strings, input/output content, runtime profiles and meaningful language test data.
- Use the new project logo in the README and integrate desktop icons only with attribution and platform verification.
- Capture genuine app images using synthetic content after the UI runs.
- Build an executable-only public preview and a separate local team package with private templates.
- Test the team package under a standard user from a different folder.
- Check package contents for keys, user data, internal documents and absolute user paths.

Exit evidence: a tested portable ZIP, accurate English documentation and a clean, reviewable Git diff. Prepare local commits and owner push instructions; do not push or publish on the owner's behalf in this handoff.

## Follow-on work

Add an authoritative document registry and an approved glossary only when supplied. Consider richer visual/direct-formatting comparison and previewed template migrations after core document fidelity is demonstrated.
