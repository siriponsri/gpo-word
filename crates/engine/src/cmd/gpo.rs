//! GPO commands shared by the desktop, CLI and MCP.
use crate::{CmdError, CmdResult, CommandSpec, Session, gpo, p};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub fn specs() -> Vec<CommandSpec> {
    vec![
        CommandSpec::new("gpo.styles", "Install GPO heading styles", "GPO", styles).params(r#"{"profile"?: Profile}"#),
        CommandSpec::new("gpo.fields", "Find unresolved GPO template fields", "GPO", fields).pure(),
        CommandSpec::new("gpo.check", "Check GPO structure and terms", "GPO", check)
            .params(r#"{"profile"?: Profile, "glossary"?: [TermRule]}"#)
            .pure(),
        CommandSpec::new("gpo.compare", "Compare GPO documents", "GPO", compare)
            .params(r#"{"original": path, "revised"?: path (otherwise current document)}"#)
            .pure(),
        CommandSpec::new("gpo.fill", "Fill GPO template fields", "GPO", fill)
            .params(r#"{"values": {field: value}} — replaces explicit {{field}} in all stories"#),
        CommandSpec::new("gpo.revision", "Append reviewed revision history", "GPO", revision)
            .params(r#"{"revision": string, "date": string, "summary": string, "author"?: string}"#),
        CommandSpec::new("gpo.thai", "Thai document utility", "GPO", thai)
            .params(r#"{"mode": "thaiDigits|arabicDigits|baht|date", "text": string, "buddhistInput"?: bool}"#)
            .pure(),
        CommandSpec::new("gpo.exportPdf", "Export GPO PDF with optional watermark", "GPO", export_pdf)
            .params(r#"{"path": string, "watermark"?: string | null, "noOverwrite"?: bool}"#)
            .pure(),
        CommandSpec::new("gpo.mergePreview", "Preview a GPO CSV merge", "GPO", merge_preview).pure(),
        CommandSpec::new("gpo.merge", "Create a GPO DOCX per CSV record", "GPO", merge)
            .params(r#"{"csv": path, "directory": existing directory, "nameField"?: string (default machineCode)}"#)
            .pure(),
    ]
}
fn export_pdf(s: &mut Session, v: &Value) -> CmdResult {
    let raw = p::req_str(v, "path")?;
    let path = if raw.to_ascii_lowercase().ends_with(".pdf") { raw.to_string() } else { format!("{raw}.pdf") };
    let mut doc = s.doc.clone();
    doc.settings.watermark = match v.get("watermark").and_then(Value::as_str) {
        Some(text) if !text.is_empty() => {
            if text.chars().count() > 200 {
                return Err(CmdError::Params("Watermark is too long".into()));
            }
            Some(wordcraft_doc::Watermark { text: text.into(), font: wordcraft_doc::styles::THAI_FONT.into(), ..Default::default() })
        }
        _ => None,
    };
    if p::bool(v, "noOverwrite").unwrap_or(false) {
        crate::io::save_path_new_with(std::path::Path::new(&path), &doc, None, &crate::io::ExportOptions::default()).map_err(CmdError::Failed)?;
    } else {
        crate::io::save_path(std::path::Path::new(&path), &doc).map_err(CmdError::Failed)?;
    }
    Ok(json!({"path":path,"sourceUnchanged":true}))
}
type MergeJobs = Vec<(std::path::PathBuf, BTreeMap<String, String>)>;

fn csv_rows(v: &Value) -> Result<(Vec<Vec<String>>, String), CmdError> {
    use sha2::{Digest, Sha256};
    let csv = p::req_str(v, "csv")?;
    crate::file_access::check_read(csv).map_err(CmdError::Failed)?;
    let bytes = crate::io::read_file_capped(csv, 10_000_000, "CSV").map_err(CmdError::Failed)?;
    let digest = Sha256::digest(&bytes).iter().map(|byte| format!("{byte:02x}")).collect::<String>();
    let text = String::from_utf8(bytes).map_err(|e| CmdError::Failed(format!("CSV is not UTF-8: {e}")))?;
    let rows = super::mailings::parse_csv(text.trim_start_matches('\u{feff}'));
    let headers = rows.first().ok_or_else(|| CmdError::Params("CSV is empty".into()))?;
    let mut unique = std::collections::BTreeSet::new();
    if headers.iter().any(|h| h.is_empty() || !unique.insert(h)) {
        return Err(CmdError::Params("CSV headers must be nonempty and unique".into()));
    }
    if rows.len() < 2 || rows.len() > 1001 {
        return Err(CmdError::Params("Use 1 to 1000 records".into()));
    }
    if rows.iter().skip(1).any(|row| row.len() != headers.len()) {
        return Err(CmdError::Params("CSV record has the wrong number of columns".into()));
    }
    Ok((rows, digest))
}
fn merge_fields(s: &Session) -> Result<Vec<String>, CmdError> {
    let findings = gpo::unresolved_fields(&s.doc).map_err(CmdError::Failed)?;
    if findings.iter().any(|finding| finding.code != "template-field") {
        return Err(CmdError::Params("Some template fields require manual correction before batch creation".into()));
    }
    Ok(findings.into_iter().map(|finding| finding.suggested).collect::<std::collections::BTreeSet<_>>().into_iter().collect())
}
fn merge_plan(s: &Session, v: &Value, rows: &[Vec<String>]) -> Result<MergeJobs, CmdError> {
    let directory = p::req_str(v, "directory")?;
    let dir = std::path::Path::new(directory);
    crate::file_access::check_write(directory).map_err(CmdError::Failed)?;
    if !dir.is_dir() {
        return Err(CmdError::Params("Choose an existing output directory".into()));
    }
    let headers = rows.first().ok_or_else(|| CmdError::Params("CSV is empty".into()))?;
    let name_field = p::str(v, "nameField").unwrap_or("machineCode");
    let name_column =
        headers.iter().position(|x| x == name_field).ok_or_else(|| CmdError::Params(format!("Missing output-name column {name_field}")))?;
    let mapping: BTreeMap<String, String> = v
        .get("mapping")
        .map(|m| serde_json::from_value(m.clone()).map_err(|e| CmdError::Params(format!("Invalid mapping: {e}"))))
        .transpose()?
        .unwrap_or_default();
    let fields = merge_fields(s)?;
    let mut columns = Vec::new();
    for field in fields {
        let column = mapping.get(&field).map(String::as_str).unwrap_or(&field);
        let index = headers
            .iter()
            .position(|header| header == column)
            .ok_or_else(|| CmdError::Params(format!("No CSV column selected for {{{{{field}}}}}")))?;
        columns.push((field, index));
    }
    let safe = regex::Regex::new(r"^[A-Za-z0-9][A-Za-z0-9._-]{0,79}$").map_err(|e| CmdError::Failed(e.to_string()))?;
    let mut names = std::collections::BTreeSet::new();
    let mut jobs = Vec::new();
    for (row_number, row) in rows.iter().enumerate().skip(1) {
        let name = row.get(name_column).ok_or_else(|| CmdError::Params("Missing output name".into()))?;
        let stem = name.split('.').next().unwrap_or("").to_ascii_uppercase();
        let reserved = matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
            || (stem.len() == 4
                && (stem.starts_with("COM") || stem.starts_with("LPT"))
                && stem.ends_with(['1', '2', '3', '4', '5', '6', '7', '8', '9']));
        if !safe.is_match(name) || name.ends_with('.') || reserved || !names.insert(name.to_ascii_lowercase()) {
            return Err(CmdError::Params(format!("Record {row_number}: output names must be unique Windows-safe machine codes")));
        }
        let path = dir.join(format!("{name}.docx"));
        crate::file_access::check_write(&path.to_string_lossy()).map_err(CmdError::Failed)?;
        if path.exists() {
            return Err(CmdError::Params(format!("Output already exists: {}", path.display())));
        }
        let mut values = BTreeMap::new();
        for (field, index) in &columns {
            let value = row.get(*index).ok_or_else(|| CmdError::Params(format!("Record {row_number}: missing column")))?;
            if value.trim().is_empty() {
                return Err(CmdError::Params(format!("Record {row_number}: empty value for {{{{{field}}}}}")));
            }
            if value.len() > 100_000 {
                return Err(CmdError::Params(format!("Record {row_number}: value is too long for {{{{{field}}}}}")));
            }
            values.insert(field.clone(), value.clone());
        }
        jobs.push((path, values));
    }
    Ok(jobs)
}
fn merge_preview(s: &mut Session, v: &Value) -> CmdResult {
    let (rows, digest) = csv_rows(v)?;
    let fields = merge_fields(s)?;
    let plan = merge_plan(s, v, &rows);
    let (ready, error, names) = match plan {
        Ok(jobs) => (
            true,
            String::new(),
            jobs.iter().take(3).map(|(path, _)| path.file_name().unwrap_or_default().to_string_lossy().into_owned()).collect::<Vec<_>>(),
        ),
        Err(error) => (false, error.to_string(), Vec::new()),
    };
    Ok(
        json!({"headers":rows.first(),"fields":fields,"sample":rows.iter().skip(1).take(3).collect::<Vec<_>>(),"records":rows.len()-1,"ready":ready,"error":error,"outputNames":names,"csvDigest":digest}),
    )
}
fn merge(s: &mut Session, v: &Value) -> CmdResult {
    let (rows, digest) = csv_rows(v)?;
    if let Some(expected) = p::str(v, "csvDigest")
        && expected != digest
    {
        return Err(CmdError::Params("CSV changed since preview; preview again".into()));
    }
    let jobs = merge_plan(s, v, &rows)?;
    let mut written = Vec::new();
    for (path, values) in jobs {
        let mut doc = s.doc.clone();
        if let Err(error) = gpo::fill_fields(&mut doc, &values) {
            return Ok(json!({"complete":false,"written":written,"error":error}));
        }
        match crate::io::save_path_new_with(&path, &doc, None, &crate::io::ExportOptions::default()) {
            Ok(()) => written.push(path),
            Err(e) => return Ok(json!({"complete":false,"written":written,"error":e})),
        }
    }
    Ok(json!({"complete":true,"written":written}))
}
fn profile(v: &Value) -> Result<gpo::Profile, CmdError> {
    v.get("profile")
        .map(|x| serde_json::from_value(x.clone()).map_err(|e| CmdError::Params(e.to_string())))
        .unwrap_or_else(|| Ok(gpo::Profile::default()))
}
fn styles(s: &mut Session, v: &Value) -> CmdResult {
    let num = gpo::install_styles(&mut s.doc, &profile(v)?).map_err(CmdError::Params)?;
    Ok(json!({"num":num,"styles":9}))
}
fn fields(s: &mut Session, _: &Value) -> CmdResult {
    let findings = gpo::unresolved_fields(&s.doc).map_err(CmdError::Params)?;
    Ok(json!({"findings":findings}))
}
fn check(s: &mut Session, v: &Value) -> CmdResult {
    let glossary = if let Some(g) = v.get("glossary") {
        serde_json::from_value(g.clone()).map_err(|e| CmdError::Params(e.to_string()))?
    } else {
        gpo::Pack::builtin().map_err(CmdError::Failed)?.glossary
    };
    let findings = gpo::validate(&s.doc, &profile(v)?, &glossary).map_err(CmdError::Params)?;
    Ok(json!({"findings":findings,"referenceExistenceChecked":false}))
}
fn compare(s: &mut Session, v: &Value) -> CmdResult {
    let original = crate::io::open_path(std::path::Path::new(p::req_str(v, "original")?)).map_err(CmdError::Failed)?;
    let revised = if let Some(path) = p::str(v, "revised") {
        crate::io::open_path(std::path::Path::new(path)).map_err(CmdError::Failed)?
    } else {
        s.doc.clone()
    };
    Ok(json!({"changes":gpo::compare(&original,&revised).map_err(CmdError::Failed)?}))
}
fn fill(s: &mut Session, v: &Value) -> CmdResult {
    if s.doc.settings.track_changes {
        return Err(CmdError::Disabled("Template filling is available before enabling Track Changes".into()));
    }
    let values: BTreeMap<String, String> =
        serde_json::from_value(v.get("values").cloned().ok_or_else(|| CmdError::Params("values is required".into()))?)
            .map_err(|e| CmdError::Params(e.to_string()))?;
    Ok(json!({"replacements":gpo::fill_fields(&mut s.doc,&values).map_err(CmdError::Params)?}))
}
fn revision(s: &mut Session, v: &Value) -> CmdResult {
    if s.doc.settings.track_changes {
        return Err(CmdError::Disabled("Turn off Track Changes before appending a reviewed history row".into()));
    }
    gpo::append_revision(
        &mut s.doc,
        p::req_str(v, "revision")?,
        p::req_str(v, "date")?,
        p::req_str(v, "summary")?,
        p::str(v, "author").unwrap_or(""),
    )
    .map_err(CmdError::Params)?;
    Ok(json!({"appended":true}))
}
fn thai(_: &mut Session, v: &Value) -> CmdResult {
    let text = p::req_str(v, "text")?;
    if text.len() > 100_000 {
        return Err(CmdError::Params("Text is too long".into()));
    }
    let text = match p::req_str(v, "mode")? {
        "thaiDigits" => gpo::digits(text, true),
        "arabicDigits" => gpo::digits(text, false),
        "baht" => gpo::baht(text).map_err(CmdError::Params)?,
        "date" => gpo::thai_date(text, p::bool(v, "buddhistInput").unwrap_or(false)).map_err(CmdError::Params)?,
        _ => return Err(CmdError::Params("Unknown Thai utility mode".into())),
    };
    Ok(json!({"text":text}))
}
