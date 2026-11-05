# Validation status

> **Expanded UX status (2026-11-05):** The current source passed all six local CI stages. Development-build UI checks and the extracted final release workflow passed as recorded below. Earlier package results are distinguished from the final release. The PDF add-on is paused.

> The GPO PDF export now supports exclusive creation. A real-app development-build check verified Thai watermark text, refusal to overwrite an existing PDF and an unchanged source document.

Evidence recorded on Windows, 2026-11-04. This is a preview, not an assertion that any template is approved for operational use.

| Check | Result | Evidence and limits |
| --- | --- | --- |
| Windows development build | PASS | Rust 1.95.0, x86_64-pc-windows-gnu; `cargo build --locked --offline -p wordcraft -p wordcraft-cli`, optimized dev profile. |
| Release build | PASS | `cargo build --release --locked --offline -j 1 -p wordcraft -p wordcraft-cli` succeeded after the CSV file-access and UI font fixes. Final portable-package workflow verification is recorded separately. |
| GPO integration tests | PASS | `cargo test -p wordcraft-engine --test gpo`: 14 passed, 0 failed, including oversized and invalid UTF-8 CSV cases. |
| Portable profile and storage tests | PASS | Four control-key/profile and four storage tests executed with `rustc --test`. |
| Local full CI | PASS | `cargo xtask ci` passed all six stages: formatting, Clippy, workspace tests, asset attribution, layering and WASM cross-compilation. Earlier runs exposed and corrected a stale help assertion, a file-access gateway violation, a font-dependent PDF test, Windows emoji fixtures and a Thai UI font fallback issue. |
| Initial GitHub Windows run | FAILED | Run 41834528624 on `a0a15ac` exposed 51 missing Thai labels in a layout that the local UI coverage test did not render. The catalog now includes those labels, and a direct regression checks them independently of visible layout. See the subsequent GitHub Actions run for its result. |
| Independent checks | PASS | Asset attribution passed for 89 assets, including the manual screenshots; architecture layering passed for 20 crates. |
| Windows startup | PASS | Initial executable crashed because named UI fonts were installed too late. Registration was moved to app creation; the rebuilt executable started and remained available through its control channel. The first-frame font regression test passed. |
| Synthetic document | PASS | Running app created, saved and reopened a synthetic DOCX. Inspection showed distinct paragraphs and linked Head 01/02 styles. CLI also saved and read back a synthetic DOCX. |
| Synthetic batch | PASS | The app generated 20 DOCX files after common metadata was filled. All 20 contained their expected machine codes and shared document number. A repeat was refused because outputs already existed. |
| PDF export | PASS with limits | Plain and DRAFT-watermarked PDFs were created, each one page; extracted text contained DRAFT only in the watermarked copy. The saved source file hash did not change. Visual fidelity and print comparison are not established. |
| Thai utility | PASS with limits | 1234.50 produced the expected Thai baht wording in one synthetic check. |
| Screenshots | PASS | Manual images were captured from the running application with synthetic content. TH Sarabun New was not installed on the test machine; a substitute font was used. |
| Private template rendering | PARTIAL | The extracted release CLI exported one DOCX and one legacy DOC to nonblank PDFs locally with source hashes unchanged. A separate read-only Word reference export stalled, so visual fidelity has not been established. The 15 copied source templates previously matched their originals by SHA-256. |
| AI provider and DPAPI | NOT RUN | No live provider call or normal-profile secure-key save/reload was performed. No API key is included. |
| Earlier development-preview ZIP | PASS | Archive integrity, required files and a sensitive-settings scan passed. Its executable launched from a fresh extraction with an isolated portable profile, both with a control channel and normally without arguments. |
| Earlier release candidate ZIP | PASS, SUPERSEDED | The candidate passed archive integrity, required-file and sensitive-settings scans, then launched from a fresh extraction with a portable profile. Its synthetic workflow produced 20 correctly populated DOCX files, refused overwrites, and exported plain and watermarked PDFs without changing the saved source. It predates the CSV and UI font fixes; rebuild and retest the final archive. |
| Corrected release package candidate | PASS | A package containing the rebuilt release binaries passed archive integrity, required-file and sensitive-settings scans. Its extracted app launched normally and with a control channel, stayed responsive and closed cleanly. The synthetic workflow produced 20 correctly populated DOCX files, refused overwrites and exported plain and watermarked PDFs without changing the saved source. One private DOCX and one DOC rendered locally to nonblank PDFs with unchanged input hashes; visual fidelity was not established. |
| Another Windows account or machine | NOT RUN | The package has not been tested under another account or on another machine. |

Reference checks validate syntax and placeholders, not the existence of authoritative records. Comparison is not a complete visual or formatting diff. Review generated output and template suitability before use. Dedicated OJT and TOR templates were not supplied.

The template pack is confidential. Do not add `docsStorage`, generated documents, local settings or keys to a public repository. The Desktop installation fixture passed without touching the actual Desktop and preserved existing sample files in the fixture.


## GPO ribbon update

The updated Windows development build launched with synthetic content. All six ribbon buttons opened distinct docked panels; light and dark appearances were inspected, and the no-document ribbon regression passed. A preliminary extracted package discovered a locally bundled template, created a separate copy, and preserved all 19 template-pack files. A synthetic batch produced 20 matching DOCX files, refused overwrites, and exported plain and watermarked PDFs without changing its source. One local DOCX and one legacy DOC rendered to nonblank PDFs with unchanged source hashes; visual fidelity was not established. The fit-width action and pane scrolling were exercised in the running app. Final-source CI passed formatting, Clippy, workspace tests, asset attribution, layering and WASM checks. The exact final ZIP was later tested; installed binaries were verified by hash as described below. The PDF organizer add-on is paused; its example could not be inspected because its Library transfer failed during required Windows metadata handling.


> The Thai watermark font and glyph-position correction was added after the preceding CI run. A Windows development build produced a visible Thai watermark and a one-page PDF from which the exact Thai text was extracted; the source file hash was unchanged by export. Fresh full CI passed formatting, Clippy, workspace tests, asset attribution, layering and WASM; the exact final package was later retested as described below.

## Expanded UX validation — 2026-11-05

- The source including the optional-path label, CSV feedback, Thai watermark correction and PDF overwrite protection passed `cargo xtask ci`: formatting, Clippy, workspace tests, asset attribution, layering and WASM. The targeted GPO tests cover unresolved fields, tracked deletions, literal field values, CSV mapping and changed-input refusal; the save test covers exclusive creation. The localhost mock verified that the outgoing AI request body matches the preview construction without placing the key in the body.
- A Windows development build opened with an isolated synthetic template pack. Actual pointer actions created and saved a new template copy with the template hash unchanged, opened and hid its text preview, and selected a document finding at body offset 20. After an edit, the check was marked stale and the jump button disabled. A separate UI test showed consent being cleared when AI context changed; no network request was made. Small-screen screenshots show the fit-width action; the prior default 100% view can clip content until the user chooses it or closes the pane.
- An independent real-app walkthrough was limited by an unfocused test window; it did not verify typing or native Save As. Its startup-font notice observation is documented in the task evidence. Native file-dialog and header/footer jump UI behavior remain unverified.
- A fresh release build of the current source succeeded with both application and CLI. The guarded cached checkout was restored and its baseline checked. The extracted release candidate and final ZIP results follow; the guarded Desktop update completed with verified hashes.

### Expanded UX package rehearsal (development binaries)

A fresh development-binary ZIP passed archive integrity, required-file and sensitive-settings scans, launched from a clean extraction, opened six distinct ribbon views, and matched all 19 private template-pack files to the local snapshot. The extracted app passed unresolved-field detection, exclusive save refusal, document comparison, CSV preview, a 20-record merge with expected contents, overwrite refusal, and plain, English-watermarked and Thai-watermarked PDF exports with the saved source unchanged. A private DOCX opened locally as an untitled template copy and was saved to a new local file with the source hash unchanged. One packaged DOCX and one legacy DOC rendered locally to nonblank PDF pages with unchanged input hashes. These are development-build rehearsal results; separate final-release results follow. Visual fidelity remains unverified.

### Extracted release candidate (current source)

The release candidate ZIP passed integrity, required-file and sensitive-settings checks. Its freshly extracted executable started, opened six distinct ribbon views and exited normally. All 19 packaged template-pack files matched the local snapshot. A synthetic workflow detected unresolved fields, refused an existing DOCX, compared revisions, previewed CSV, generated 20 correct DOCX files and refused collisions. Plain, English-watermarked and Thai-watermarked PDFs were produced; Thai text was extractable, an existing PDF was preserved, and the saved source remained unchanged.

One packaged private DOCX opened as a new copy and saved locally without changing the source. The release CLI rendered one DOCX and one legacy DOC locally to nonblank pages with source hashes unchanged. These checks do not establish visual fidelity against Microsoft Word. No private content was uploaded. Live AI provider calls, DPAPI behavior under another Windows account, native file-dialog interaction, header/footer jump interaction and use on another machine were not tested. The final ZIP was subsequently retested after assembly, as recorded below.

### Final extracted ZIP and local installation

The final archive passed integrity, required-file and sensitive-settings scans and matched all 19 template-pack files to the private local snapshot. Its extracted application launched, displayed six distinct ribbon views, used a portable profile and exited normally. The extracted workflow detected unresolved fields, compared revisions, refused duplicate DOCX and PDF files, generated and checked 20 DOCX outputs, and exported ordinary and Thai-watermarked PDF without changing the saved source. A packaged private DOCX opened and saved as a new local copy with its source unchanged. One DOCX and one legacy DOC rendered locally to nonblank pages with unchanged source hashes.

The locally installed executable, CLI, README, validation document, Thai manual and ZIP matched the tested package by hash; protected existing files and samples remained unchanged. The installed executable itself was not launched in place: identical bytes were launched from the extracted ZIP. The final archive SHA-256 is `af035bb14ca9b6b3a4f17e8c4098c08913ad28f4f45094c12e96b4424c340828`. Live AI provider calls, another Windows account or computer, native file-picker interaction, header/footer finding navigation and visual comparison against Word remain untested.
