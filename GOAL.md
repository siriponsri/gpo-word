# Product goals

GPO Word helps Windows users prepare consistent Thai pharmaceutical documents with less repeated formatting and transcription. Users retain control of technical facts, templates, wording and approval decisions.

## Intended outcome

A teammate can receive a portable package, select the appropriate private template, draft or edit a document, check common mistakes, export a review copy and save an editable DOCX without installing development tools.

Success is measured by document outcomes and reproducible evidence, not by time spent or a claim that source files exist.

## Product principles

1. The selected template governs layout. Profiles govern structure and checks; terminology rules govern vocabulary; writing guidance governs drafting.
2. Users can write every part themselves. AI is optional and limited to reviewed content and explicit tasks.
3. Heading numbers are real linked paragraph numbering, not manually typed prefixes.
4. Common format and wording changes are editable without changing application code.
5. Comparison reports preserve evidence, and revision history keeps existing rows.
6. Batch generation protects existing files and identifies partial failures.
7. Template packs travel between Windows accounts and machines without hard-coded user paths.
8. Public code and documentation are separated from private templates, documents and credentials.

## Document scope

WI, QP, FM, LG, SP, document indexes, training questionnaires, OJT, TOR and internal memoranda are in scope. Logbook templates primarily define the cover; users design subsequent tables. Memorandum templates define headers; reusable writing patterns do not copy past subjects, recipients or signatures.

OJT and TOR have no dedicated source template in the supplied pack. Do not silently substitute an examination template or invent an approved format.

## Completion evidence

- The supported Windows build and the repository's required checks pass.
- Representative template documents survive open, edit, save/reopen, Word round trip and PDF export without loss of critical content or structure.
- Head 01 and Head 02 insertions/deletions renumber correctly before and after reopening.
- A revision summary reflects the actual old/new pair; appending it preserves existing history.
- A 20-record synthetic FM/LG batch produces exactly one correctly populated output per record.
- Optional AI remains usable without plaintext key storage and handles offline, authentication and rate-limit errors.
- The portable package starts as a standard Windows user in a second location and contains no previous user's data or keys.
- Documentation, validation results and release claims match observed behavior.

A lack of API credentials may block the live API check. It must not stop local document editing, deterministic tests, packaging work or clear reporting of remaining gaps.

## Technical content integrity

Preserve document identifiers, revisions, dates, values, units, formulae, acceptance criteria and named responsibilities unless the user explicitly requests a technical change. Unknown information remains a visible placeholder. Neither a model output nor a syntax check establishes approval, GMP compliance or current document validity.

## Delivery boundary

The target of this handoff is a complete, tested Windows proof of concept with a reviewable portable package. Formal adoption, validation and controlled-document approval remain organization processes.
