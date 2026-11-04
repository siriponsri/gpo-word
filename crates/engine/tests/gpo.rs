use serde_json::json;
use std::{collections::BTreeMap, sync::Arc};
use wordcraft_doc::{Block, CharProps, Document, Paragraph, StoryRef, para_block};
use wordcraft_doc::{Cell, PartKind, Row, Table};
use wordcraft_engine::{Session, gpo::*};

#[test]
fn builtin_pack_has_each_document_family() {
    let p = Pack::builtin().unwrap();
    for id in ["WI", "QP", "FM", "LG", "SP", "INDEX", "EXAM", "OJT", "MEMO", "TOR"] {
        assert!(p.profiles.iter().any(|p| p.id == id));
    }
}
#[test]
fn numbering_is_linked_idempotent_and_survives_docx() {
    let mut d = Document::from_text("วัตถุประสงค์\nรายละเอียด\nขอบเขต\nรายละเอียดต่อ");
    let num = install_styles(&mut d, &Profile::default()).unwrap();
    assert_eq!(install_styles(&mut d, &Profile::default()).unwrap(), num);
    for (i, level) in [0, 1, 0, 1].iter().enumerate() {
        let path = wordcraft_doc::Path(vec![i as u32]);
        d.para_mut(StoryRef::Body, &path).unwrap().props.style = Some(format!("GPOHead{:02}", level + 1));
    }
    let bytes = wordcraft_docx::write(&d).unwrap();
    let read = wordcraft_docx::read(&bytes).unwrap();
    let style = read.styles.find("Head 02").unwrap();
    assert_eq!(style.para.outline_level, Some(1));
    assert_eq!(style.para.numbering.unwrap().level, 1);
    let numbering = style.para.numbering.unwrap();
    let linked = read.numbering.level(numbering.num, numbering.level).unwrap();
    assert_eq!(linked.text, "%1.%2");
    assert_eq!(read.styles.styles.iter().filter(|s| s.name.starts_with("Head ")).count(), 9);
}
#[test]
fn fill_split_runs_preserves_headers_and_surrounding_format() {
    let mut d = Document::new();
    let mut p = Paragraph::with_text("รหัส {{machineCode}} จบ", CharProps { bold: Some(true), ..Default::default() });
    let split = p.text.find("machineCode").unwrap() + 4;
    p.format(split, split + 3, &|c| c.italic = Some(true)).unwrap();
    d.body = vec![para_block(p)];
    let h = d.add_part(PartKind::Header, vec![para_block(Paragraph::with_text("{{machineCode}}", Default::default()))]);
    assert_eq!(fill_fields(&mut d, &BTreeMap::from([("machineCode".into(), "MC-020".into())])).unwrap(), 2);
    assert_eq!(d.final_text(StoryRef::Body), "รหัส MC-020 จบ");
    assert_eq!(d.final_text(StoryRef::Part(h)), "MC-020");
    assert_eq!(d.body.first().unwrap().as_para().unwrap().runs.first().unwrap().props.bold, Some(true));
}
#[test]
fn checks_distinguish_wi_and_qp_and_find_typos_in_tables() {
    let pack = Pack::builtin().unwrap();
    let wi = pack.profiles.iter().find(|p| p.id == "WI").unwrap();
    let qp = pack.profiles.iter().find(|p| p.id == "QP").unwrap();
    let mut d = Document::from_text("1. วัตถุประสงค์\n2. ขอบเขต\n3. ผู้ปฏิบัติงาน\n5. วิธีปฏิบัติงาน");
    d.body.push(Arc::new(Block::Table(Table {
        rows: vec![Row { cells: vec![Cell::with_text("ปฎิบัติตาม WI-ABCD-XXX")], ..Default::default() }],
        ..Default::default()
    })));
    let issues = validate(&d, wi, &pack.glossary).unwrap();
    assert!(!issues.iter().any(|f| f.code == "missing-heading"));
    assert!(issues.iter().any(|f| f.suggested == "ปฏิบัติ"));
    assert!(issues.iter().any(|f| f.code == "reference-format"));
    assert!(validate(&d, qp, &[]).unwrap().iter().any(|f| f.suggested == "ความรับผิดชอบ"));
}
#[test]
fn compare_catches_header_and_style_only_changes() {
    let mut d = Document::from_text("เนื้อหาคงเดิม");
    d.add_part(PartKind::Header, vec![para_block(Paragraph::with_text("Rev. 01", Default::default()))]);
    let mut n = d.clone();
    let id = *n.parts.keys().next().unwrap();
    n.parts.get_mut(&id).unwrap().blocks = vec![para_block(Paragraph::with_text("Rev. 02", Default::default()))];
    assert!(compare(&d, &n).unwrap().iter().any(|c| c.section.starts_with("Header/")));
    n.styles.default_chr.size = Some(18.0);
    assert!(compare(&d, &n).unwrap().iter().any(|c| c.section == "styles"));
}
#[test]
fn revision_is_appended_without_overwriting_previous_rows() {
    let mut d = Document::new();
    let t = Table {
        rows: vec![
            Row {
                cells: vec![Cell::with_text("แก้ไขครั้งที่"), Cell::with_text("วันที่บังคับใช้"), Cell::with_text("รายละเอียดการเปลี่ยนแปลง")],
                ..Default::default()
            },
            Row {
                cells: vec![Cell::with_text("01"), Cell::with_text("1 ตุลาคม 2569"), Cell::with_text("ประกาศใช้ครั้งแรก")], ..Default::default()
            },
        ],
        ..Default::default()
    };
    d.body.push(Arc::new(Block::Table(t)));
    append_revision(&mut d, "02", "2 พฤศจิกายน 2569", "แก้ไขหัวเอกสาร", "").unwrap();
    let t = d.body.last().unwrap().as_table().unwrap();
    assert_eq!(t.rows.len(), 3);
    assert_eq!(t.rows[1].cells[0].blocks[0].as_para().unwrap().text, "01");
    assert_eq!(t.rows[2].cells[2].blocks[0].as_para().unwrap().text, "แก้ไขหัวเอกสาร");
}
#[test]
fn memo_uses_header_and_removes_old_body_and_signer() {
    let mut d = Document::from_text("บันทึกข้อความ\n\nหน่วยงานเก่า\nที่ เลขเดิม วันที่เดิม\nเรื่องเดิม\nเนื้อหาเก่า\nผู้ลงนามเดิม");
    let p = Profile { memo_header_only: true, ..Default::default() };
    prepare_template(&mut d, &p).unwrap();
    let s = d.final_text(StoryRef::Body);
    assert!(s.contains("{{title}}"));
    assert!(!s.contains("เนื้อหาเก่า"));
    assert!(!s.contains("ผู้ลงนามเดิม"));
}

#[test]
fn portable_memo_preset_uses_header_fields_and_keeps_page_geometry() {
    let preset = json!({"schemaVersion":1,"kind":"memo-header","section":{"pageW":595.3,"pageH":841.9,"marginLeft":85.05},
        "paragraphs":[{"text":"บันทึกข้อความ","chr":{},"para":{}},{"text":"หน่วยงาน {{department}}","chr":{},"para":{}},
        {"text":"ที่ {{documentNumber}} วันที่ {{date}}","chr":{},"para":{}},{"text":"เรื่อง {{title}}","chr":{},"para":{}}]});
    let mut doc = memo_from_blueprint(preset.to_string().as_bytes()).unwrap();
    prepare_template(&mut doc, &Profile { memo_header_only: true, ..Default::default() }).unwrap();
    assert_eq!(doc.last_section.margin_left, 85.05);
    assert!(doc.final_text(StoryRef::Body).contains("{{recipient}}"));
}
#[test]
fn thai_calendar_validates_leap_days_and_explicit_buddhist_input() {
    assert_eq!(thai_date("2026-11-02", false).unwrap(), "2 พฤศจิกายน 2569");
    assert_eq!(thai_date("2569-11-02", true).unwrap(), "2 พฤศจิกายน 2569");
    assert!(thai_date("2025-02-29", false).is_err());
    assert!(thai_date("2024-02-29", false).is_ok());
    for s in ["2026-00-01", "2026-13-01", "2026-01-00", "2026-04-31", ""] {
        assert!(thai_date(s, false).is_err());
    }
}
#[test]
fn money_uses_integer_arithmetic_and_thai_exceptions() {
    for (input, expected) in [
        ("0", "ศูนย์บาทถ้วน"),
        ("1", "หนึ่งบาทถ้วน"),
        ("11", "สิบเอ็ดบาทถ้วน"),
        ("21.25", "ยี่สิบเอ็ดบาทยี่สิบห้าสตางค์"),
        ("101.01", "หนึ่งร้อยเอ็ดบาทหนึ่งสตางค์"),
        ("1000001", "หนึ่งล้านเอ็ดบาทถ้วน"),
        ("0.5", "ศูนย์บาทห้าสิบสตางค์"),
    ] {
        assert_eq!(baht(input).unwrap(), expected);
    }
    for input in ["-1", "NaN", "1.999", "1,000", "1000000000000", "1e3", ".50"] {
        assert!(baht(input).is_err());
    }
}
#[test]
fn digit_conversion_round_trips_without_touching_english() {
    let text = "FM-ANF3-001 2569/11";
    assert_eq!(digits(&digits(text, true), false), text);
}
#[test]
fn gpo_commands_are_registered_and_styles_are_undoable() {
    let mut s = Session::new(Document::from_text("ข้อความเดิม"));
    let before = s.doc.clone();
    s.run("gpo.styles", &json!({})).unwrap();
    assert!(s.doc.styles.find("Head 01").is_some());
    s.run("edit.undo", &json!({})).unwrap();
    assert_eq!(s.doc, before);
    assert_eq!(s.run("gpo.thai", &json!({"mode":"date","text":"2026-11-02"})).unwrap()["text"], "2 พฤศจิกายน 2569");
}
#[test]
fn hostile_profile_and_prompt_inputs_are_refused() {
    assert!(Pack::parse("{}").is_err());
    let p = Profile::default();
    assert!(prompt(&p, &"x".repeat(120_001), "review").is_err());
    let mut d = Document::new();
    assert!(install_styles(&mut d, &Profile { font_size: f32::NAN, ..Default::default() }).is_err());
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn batch_csv_rejects_oversized_and_invalid_utf8_files() {
    let dir = std::env::temp_dir().join(format!("gpo-merge-csv-{}", std::process::id()));
    std::fs::create_dir(&dir).unwrap();
    let csv = dir.join("input.csv");
    std::fs::File::create(&csv).unwrap().set_len(10_000_001).unwrap();
    let mut session = Session::new(Document::new());
    let params = json!({"csv": csv, "directory": dir});
    assert!(session.run("gpo.merge", &params).is_err());
    std::fs::write(&csv, [0xff]).unwrap();
    assert!(session.run("gpo.merge", &params).is_err());
    std::fs::remove_file(csv).unwrap();
    std::fs::remove_dir(dir).unwrap();
}
