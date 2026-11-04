//! GPO commands shared by the desktop, CLI and MCP.
use crate::{CmdError, CmdResult, CommandSpec, Session, gpo, p};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub fn specs() -> Vec<CommandSpec> {
    vec![
        CommandSpec::new("gpo.styles", "Install GPO heading styles", "GPO", styles).params(r#"{"profile"?: Profile}"#),
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
            .params(r#"{"path": string, "watermark"?: string | null}"#)
            .pure(),
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
            Some(wordcraft_doc::Watermark { text: text.into(), ..Default::default() })
        }
        _ => None,
    };
    crate::io::save_path(std::path::Path::new(&path), &doc).map_err(CmdError::Failed)?;
    Ok(json!({"path":path,"sourceUnchanged":true}))
}
fn merge(s: &mut Session, v: &Value) -> CmdResult {
    let csv = p::req_str(v, "csv")?;
    let directory = p::req_str(v, "directory")?;
    let name_field = p::str(v, "nameField").unwrap_or("machineCode");
    crate::file_access::check_read(csv).map_err(CmdError::Failed)?;
    crate::file_access::check_write(directory).map_err(CmdError::Failed)?;
    let dir = std::path::Path::new(directory);
    if !dir.is_dir() {
        return Err(CmdError::Params("Choose an existing output directory".into()));
    }
    let bytes = crate::io::read_file_capped(csv, 10_000_000, "CSV").map_err(CmdError::Failed)?;
    let text = String::from_utf8(bytes).map_err(|e| CmdError::Failed(format!("CSV is not UTF-8: {e}")))?;
    let rows = super::mailings::parse_csv(text.trim_start_matches('\u{feff}'));
    let headers = rows.first().ok_or_else(|| CmdError::Params("CSV is empty".into()))?;
    let mut unique = std::collections::BTreeSet::new();
    if headers.iter().any(|h| h.is_empty() || !unique.insert(h)) {
        return Err(CmdError::Params("CSV headers must be nonempty and unique".into()));
    }
    let name_column = headers.iter().position(|x| x == name_field).ok_or_else(|| CmdError::Params(format!("Missing column {name_field}")))?;
    if rows.len() < 2 || rows.len() > 1001 {
        return Err(CmdError::Params("Use 1 to 1000 records".into()));
    }
    let placeholder = regex::Regex::new(r"\{\{([^{}]+)\}\}").map_err(|e| CmdError::Failed(e.to_string()))?;
    let source = gpo::document_text(&s.doc);
    for cap in placeholder.captures_iter(&source) {
        if let Some(field) = cap.get(1)
            && !headers.iter().any(|h| h == field.as_str())
        {
            return Err(CmdError::Params(format!("CSV has no column for {{{{{}}}}}; fill common metadata first", field.as_str())));
        }
    }
    let safe = regex::Regex::new(r"^[A-Za-z0-9][A-Za-z0-9._-]{0,79}$").map_err(|e| CmdError::Failed(e.to_string()))?;
    let mut names = std::collections::BTreeSet::new();
    let mut jobs = Vec::new();
    for row in rows.iter().skip(1) {
        if row.len() != headers.len() {
            return Err(CmdError::Params("CSV record has the wrong number of columns".into()));
        }
        let name = row.get(name_column).ok_or_else(|| CmdError::Params("Missing output name".into()))?;
        let stem = name.split('.').next().unwrap_or("").to_ascii_uppercase();
        let reserved = matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
            || (stem.len() == 4
                && (stem.starts_with("COM") || stem.starts_with("LPT"))
                && stem.ends_with(['1', '2', '3', '4', '5', '6', '7', '8', '9']));
        if !safe.is_match(name) || name.ends_with('.') || reserved || !names.insert(name.to_ascii_lowercase()) {
            return Err(CmdError::Params("Output names must be unique Windows-safe machine codes".into()));
        }
        let path = dir.join(format!("{name}.docx"));
        crate::file_access::check_write(&path.to_string_lossy()).map_err(CmdError::Failed)?;
        if path.exists() {
            return Err(CmdError::Params(format!("Output already exists: {}", path.display())));
        }
        let values = headers.iter().cloned().zip(row.iter().cloned()).collect::<BTreeMap<_, _>>();
        jobs.push((path, values));
    }
    let mut written = Vec::new();
    for (path, values) in jobs {
        let mut doc = s.doc.clone();
        gpo::fill_fields(&mut doc, &values).map_err(CmdError::Failed)?;
        match crate::io::save_path(&path, &doc) {
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
