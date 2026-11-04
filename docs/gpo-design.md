# GPO Word Windows preview

The desktop workbench extends the KhianKrub source snapshot named in `gpo/upstreams.lock.json`.
The document model, DOCX importer/exporter, typography, mailings and PDF renderer remain the upstream implementations.
GPO rules and utilities live in `crates/engine/src/gpo.rs`; their registered commands live in `cmd/gpo.rs`.
The desktop workbench and Windows DPAPI/API adapter live in `apps/wordcraft/src/gpo.rs`.

## Templates and styles

Templates stay in the user's docsStorage. File New with `template: gpo` reads a copy and leaves the save target unset.
Head 01 through Head 09 are real paragraph styles linked to one decimal multilevel list and outline levels 0 through 8.
The first level uses `1.` and deeper levels use `1.1`, `1.1.1`, with no manually typed numbers.
The profile's font and size are editable in the workbench. Existing template page sizes and headers are retained.
New policy settings are saved per user, separately from document content.

No-code field binding replaces selected sample text with `{{field}}`; the result is saved as a new DOCX or DOTX template.
The initial import does not infer arbitrary legacy template fields. It provides an explicit binding workflow.
Memo templates locate the title, department, number/date and subject labels, allowing blank paragraphs between them, and replace variable values with fields.
The old memo body and signatory are removed. Unsupported memo layouts return an error.
Logbook covers are templates; the user creates following tables with the standard table tools.

## AI and language

Typhoon Chat Completions is the default provider. The user can change the HTTPS base URL and model.
No API request happens until the user presses the action. Document requests additionally require consent to the displayed editable context.
Windows keys use DPAPI CurrentUser through a hidden PowerShell helper, passed through stdin, not command-line arguments.
Saved keys are tied to their exact base URL. On other platforms only a session key is supported in this preview.
There is no paid fallback, automatic retry or implicit whole-folder upload.
The writing skill and profile guidance are loaded by the application. Responses are editable proposals before insertion.

## Review and export

Structure checks use per-profile required headings. Terminology rules are deterministic string rules editable in Settings.
Reference checks validate syntax and obvious template placeholders; they do not assert existence or current approval.
Comparison covers final text in the body, tables, headers, footers, notes and text boxes plus styles, numbering and final page setup.
Part ids are compared by kind and occurrence. This is not a full layout diff: direct run formatting, drawings and every section property are not fully compared.
The AI summary is generated from displayed differences. A reviewed history row is appended to an existing recognized table; older rows are retained.
GPO PDF export uses a cloned document and leaves the editor's watermark settings unchanged.
CSV batch output validates all records and names first, refuses existing output paths, fills explicit fields and writes one DOCX per record.
If writing fails, the response identifies completed files. Excel/MERGEFIELD workflows remain available through upstream Mailings.

## Validation

`cargo test -p wordcraft-engine --test gpo` tests DOCX round trips, numbering, template fill, revision history, memo sanitization, comparison and Thai utilities.
`cargo xtask ci` remains the required upstream format/lint/test/assets/layers/WASM gate.
The Windows GitHub Actions workflow also builds portable binaries. A source-only delivery must not be described as a verified Windows application until these checks and a native opening test pass.
Private GPO files are excluded from Git and from portable packages.

## Portable template packs

docsStorage/catalog.json lists templates by profile and relative path. docsStorage/pack.json supplies defaults for a template pack. A memo-header.gpo.json preset carries only generic header placeholders, the source logo and page/paragraph properties, without the original memo body or signers. It is an application layout preset rather than a DOCX file.
The app discovers docsStorage next to the executable, from a saved valid location, from the working directory or nearby source/build ancestors. It no longer assumes a particular Windows username or Desktop folder. Invalid old absolute locations fall back to local discovery. Selected templates inside the pack are saved as relative paths.
GPOWord.portable beside the executable keeps GPO settings under data. Runtime data, API keys and completed documents are not included in Package-Portable output. The script needs a genuine release build before it can create a runnable archive.
