# GPO architecture

The workbench in apps/wordcraft/src/gpo.rs presents GPO document workflows and optional AI settings. Portable configuration and template discovery live in gpo_storage.rs.

The engine owns document semantics in crates/engine/src/gpo.rs and registered commands in crates/engine/src/cmd/gpo.rs. Maintain the upstream layering rule: lower document layers do not depend on the desktop UI or network-provider libraries.

## Data boundaries

- Default profiles, synthetic examples and writing guidance are public source.
- Private template catalogs contain relative paths, profile identifiers and template digests.
- User documents and organization templates remain local private data.
- API settings and DPAPI-protected keys belong to the current Windows user.
- AI receives only text the author explicitly reviews for the requested task.

## Formatting

Head 01–09 are paragraph styles linked to one multilevel numbering scheme and outline levels. Paragraph edits must preserve this relationship through DOCX serialization. A visually similar typed prefix is not an acceptable replacement.

Template profile edits and header/table layout edits are distinct operations. Updating a profile does not automatically migrate an existing document to a new header.

## Review and export

Text and selected style comparison generate evidence for a reviewed revision summary. They are not a complete visual or direct-formatting diff.

PDF export applies a watermark to a copy. Batch generation validates input fields and output names before writing; existing files remain protected. Partial failures need actionable reporting.

## Continuation

Read HANDOFF.md and the feature acceptance matrix in docs/CODEX-PROMPT.md before extending these modules. The source implementation has not yet been compiled in this handoff.
