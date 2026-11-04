# Validation status

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
