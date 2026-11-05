//! GPO ribbon and docked task pane. Document operations use the normal engine command path.
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;
use wordcraft_engine::{
    gpo::{self, Pack, Profile},
    io::Password,
};
use wordcraft_ui_egui::WordApp;

pub struct Panel {
    open: bool,
    tab: usize,
    profile: usize,
    pack: Option<Pack>,
    root: String,
    catalog: Vec<PathBuf>,
    selected_template: usize,
    template_query: String,
    template_preview: Option<(PathBuf, Result<String, String>)>,
    template_pending: Option<(PathBuf, mpsc::Receiver<Result<String, String>>)>,
    message: String,
    findings: Vec<gpo::Finding>,
    changes: Vec<gpo::Change>,
    fields: BTreeMap<String, String>,
    field_drafts: BTreeMap<u64, BTreeMap<String, String>>,
    field_doc: Option<u64>,
    field_mark: Option<(u64, u64)>,
    active_fields: Vec<String>,
    manual_fields: usize,
    checked_mark: Option<(u64, u64)>,
    checked_name: String,
    exported_mark: Option<(u64, u64)>,
    comparison_mark: Option<(u64, u64)>,
    comparison_source: String,
    revision_doc: Option<u64>,
    utility: String,
    utility_output: String,
    utility_mode: usize,
    buddhist_input: bool,
    watermark: String,
    watermark_on: bool,
    api_url: String,
    model: String,
    api_key: Password,
    task: String,
    context: String,
    answer: String,
    revision_summary: String,
    consent: bool,
    consent_receipt: Option<(String, String)>,
    pending: Option<mpsc::Receiver<Result<String, String>>>,
    revision: String,
    revision_date: String,
    revision_author: String,
    heading_lines: String,
    active_profile_editor: Option<usize>,
    request_batch: bool,
    merge_source: String,
    merge_output: String,
    merge_name_field: String,
    merge_mapping: BTreeMap<String, String>,
    merge_preview: Option<Value>,
    merge_signature: String,
}
impl Default for Panel {
    fn default() -> Self {
        let mut panel = Self {
            open: false,
            tab: 0,
            profile: 0,
            pack: Pack::builtin().ok(),
            root: String::new(),
            catalog: Vec::new(),
            selected_template: 0,
            template_query: String::new(),
            template_preview: None,
            template_pending: None,
            message: String::new(),
            findings: Vec::new(),
            changes: Vec::new(),
            fields: BTreeMap::new(),
            field_drafts: BTreeMap::new(),
            field_doc: None,
            field_mark: None,
            active_fields: Vec::new(),
            manual_fields: 0,
            checked_mark: None,
            checked_name: String::new(),
            exported_mark: None,
            comparison_mark: None,
            comparison_source: String::new(),
            revision_doc: None,
            utility: String::new(),
            utility_output: String::new(),
            utility_mode: 0,
            buddhist_input: false,
            watermark: "ฉบับร่าง".into(),
            watermark_on: true,
            api_url: "https://api.opentyphoon.ai/v1".into(),
            model: "typhoon-v2.5-30b-a3b-instruct".into(),
            api_key: Password::default(),
            task: "ปรับภาษาให้เป็นทางการและอ่านเข้าใจง่าย โดยรักษาความหมายเดิม".into(),
            context: String::new(),
            answer: String::new(),
            revision_summary: String::new(),
            consent: false,
            consent_receipt: None,
            pending: None,
            revision: String::new(),
            revision_date: String::new(),
            revision_author: String::new(),
            heading_lines: String::new(),
            active_profile_editor: None,
            request_batch: false,
            merge_source: String::new(),
            merge_output: String::new(),
            merge_name_field: "machineCode".into(),
            merge_mapping: BTreeMap::new(),
            merge_preview: None,
            merge_signature: String::new(),
        };
        let executable = std::env::current_exe().unwrap_or_default();
        let cwd = std::env::current_dir().unwrap_or_default();
        let mut saved_root = None;
        let mut saved_pack = None;
        if let Ok(dir) = config_dir() {
            if let Ok(text) = read_bounded(&dir.join("pack.json"), 2_000_000) {
                match Pack::parse(&text) {
                    Ok(pack) => saved_pack = Some(pack),
                    Err(e) => panel.message = format!("ใช้กฎเริ่มต้น เพราะอ่าน pack.json ไม่ได้: {e}"),
                }
            }
            if let Ok(text) = read_bounded(&dir.join("settings.json"), 64_000)
                && let Ok(v) = serde_json::from_str::<Value>(&text)
            {
                saved_root = v.get("root").and_then(Value::as_str).map(PathBuf::from);
                for (key, target) in [("apiUrl", &mut panel.api_url), ("model", &mut panel.model)] {
                    if let Some(s) = v.get(key).and_then(Value::as_str) {
                        *target = s.into();
                    }
                }
            }
        }
        panel.root = crate::gpo_storage::discover_root(&executable, &cwd, saved_root.as_deref()).to_string_lossy().into();
        if let Ok(text) = read_bounded(&Path::new(&panel.root).join("pack.json"), 2_000_000) {
            match Pack::parse(&text) {
                Ok(pack) => panel.pack = Some(pack),
                Err(e) => panel.message = format!("อ่านชุดเทมเพลตไม่ได้: {e}"),
            }
        }
        if let Some(pack) = saved_pack {
            panel.pack = Some(pack);
        }
        panel.rescan();
        panel
    }
}
const NAV: [(&str, &str, &[u8]); 6] = [
    ("เอกสาร", "documents", include_bytes!("../../../assets/icons/gpo/documents.png")),
    ("ตรวจทาน", "review", include_bytes!("../../../assets/icons/gpo/review.png")),
    ("AI", "ai", include_bytes!("../../../assets/icons/gpo/ai.png")),
    ("เทมเพลต", "templates", include_bytes!("../../../assets/icons/gpo/templates.png")),
    ("เครื่องมือ", "tools", include_bytes!("../../../assets/icons/gpo/tools.png")),
    ("ตั้งค่า", "settings", include_bytes!("../../../assets/icons/gpo/settings.png")),
];

impl wordcraft_ui_egui::RibbonExtension for Panel {
    fn tab_name(&self) -> &str {
        "gpo:host"
    }

    fn tab_label(&self) -> &str {
        "GPO"
    }

    fn ribbon(&mut self, app: &mut WordApp, ui: &mut egui::Ui) {
        ui.allocate_ui_with_layout(egui::vec2(100.0, 72.0), egui::Layout::top_down(egui::Align::Center), |ui| {
            ui.add_space(20.0);
            ui.menu_button("เริ่มงาน ▾", |ui| {
                if ui.button("สร้างจากเทมเพลต").clicked() {
                    self.tab = 0;
                    self.open = true;
                    ui.close();
                }
                if ui.button("เปิดเอกสาร…").clicked() {
                    self.result(app.run("file.open", json!({})));
                    ui.close();
                }
                if ui.add_enabled(app.has_document(), egui::Button::new("ตรวจเอกสาร")).clicked() {
                    self.tab = 1;
                    self.open = true;
                    self.run_check(app);
                    ui.close();
                }
                if ui.button("สร้างหลายเอกสาร").clicked() {
                    self.tab = 0;
                    self.open = true;
                    self.request_batch = true;
                    ui.close();
                }
            });
        });
        let tokens = wordcraft_ui_egui::theme::Tokens::get(ui.ctx());
        for (index, (label, name, bytes)) in NAV.iter().enumerate() {
            let (rect, response) = ui.allocate_exact_size(egui::vec2(74.0, 72.0), egui::Sense::click());
            let selected = self.open && self.tab == index;
            response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::SelectableLabel, true, selected, *label));
            if selected || response.hovered() {
                ui.painter().rect_filled(rect, 5.0, tokens.hover);
            }
            let id = egui::Id::new(("gpo-nav-icon", *name));
            let texture = ui.ctx().data(|d| d.get_temp::<egui::TextureHandle>(id)).or_else(|| {
                let image = image::load_from_memory(bytes).ok()?.to_rgba8();
                let size = [image.width() as usize, image.height() as usize];
                let pixels = egui::ColorImage::from_rgba_unmultiplied(size, image.as_raw());
                let texture = ui.ctx().load_texture(*name, pixels, egui::TextureOptions::LINEAR);
                ui.ctx().data_mut(|d| d.insert_temp(id, texture.clone()));
                Some(texture)
            });
            if let Some(texture) = texture {
                let icon = egui::Rect::from_center_size(egui::pos2(rect.center().x, rect.top() + 21.0), egui::vec2(32.0, 32.0));
                ui.painter().image(texture.id(), icon, egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)), tokens.text);
            }
            ui.painter().text(
                egui::pos2(rect.center().x, rect.top() + 49.0),
                egui::Align2::CENTER_CENTER,
                *label,
                wordcraft_ui_egui::theme::regular(12.0),
                tokens.text,
            );
            if response.clicked() {
                self.tab = index;
                self.open = true;
            }
        }
        ui.allocate_ui_with_layout(egui::vec2(125.0, 72.0), egui::Layout::top_down(egui::Align::Center), |ui| {
            ui.add_space(18.0);
            if ui.add_enabled(app.has_document(), egui::Button::new("พอดีความกว้าง")).on_hover_text("ปรับหน้ากระดาษให้พอดีกับพื้นที่แก้ไข").clicked()
                && let Err(error) = app.run("view.pageWidth", json!({}))
            {
                self.message = error;
            }
        });
    }

    fn pane(&mut self, app: &mut WordApp, ui: &mut egui::Ui) {
        if !self.open || app.ui.tab != "gpo:host" {
            return;
        }
        egui::Panel::right("gpo_task_pane").resizable(true).default_size(320.0).size_range(280.0..=600.0).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading(NAV.get(self.tab).map(|item| item.0).unwrap_or("GPO"));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("×").on_hover_text("ปิดแผง").clicked() {
                        self.open = false;
                    }
                });
            });
            ui.small("แนวทางเลือกได้: เทมเพลต → ตรวจ → ส่งออก PDF");
            if app.has_document() {
                let mark = Self::document_mark(app);
                ui.label(format!(
                    "{} · {}",
                    Self::document_name(app),
                    if app.session.path.is_none() {
                        "ยังไม่มีไฟล์ที่บันทึก"
                    } else if app.has_unsaved_changes() {
                        "มีการแก้ไขที่ยังไม่บันทึก"
                    } else {
                        "บันทึกแล้ว"
                    }
                ));
                ui.small(format!(
                    "ตรวจ: {}  ·  PDF: {}",
                    if self.checked_mark == Some(mark) {
                        "ล่าสุด"
                    } else if self.checked_mark.is_some() {
                        "ควรตรวจใหม่"
                    } else {
                        "ยังไม่ตรวจ"
                    },
                    if self.exported_mark == Some(mark) {
                        "ส่งออกแล้ว"
                    } else {
                        "ยังไม่ส่งออกฉบับนี้"
                    }
                ));
            }
            ui.separator();
            self.show_content(ui, app);
        });
    }
}

impl Panel {
    fn current(&self) -> Option<Profile> {
        self.pack.as_ref()?.profiles.get(self.profile).cloned()
    }
    fn rescan(&mut self) {
        self.catalog.clear();
        self.selected_template = 0;
        self.template_preview = None;
        self.template_pending = None;
        if let Some(p) = self.current() {
            let root = Path::new(&self.root);
            let mut used_catalog = false;
            if let Ok(text) = read_bounded(&root.join("catalog.json"), 2_000_000)
                && let Ok(value) = serde_json::from_str::<Value>(&text)
                && let Some(rows) = value.get("templates").and_then(Value::as_array)
            {
                used_catalog = true;
                for row in rows {
                    if row.get("profileId").and_then(Value::as_str) == Some(p.id.as_str())
                        && let Some(path) = row.get("path").and_then(Value::as_str).and_then(|s| crate::gpo_storage::resolve_template(root, s))
                    {
                        self.catalog.push(path);
                    }
                }
            }
            if !used_catalog
                && !p.folder.is_empty()
                && let Ok(rows) = std::fs::read_dir(root.join(&p.folder))
            {
                self.catalog = rows
                    .filter_map(Result::ok)
                    .map(|x| x.path())
                    .filter(|p| {
                        p.is_file()
                            && p.extension()
                                .and_then(|e| e.to_str())
                                .is_some_and(|e| matches!(e.to_ascii_lowercase().as_str(), "doc" | "docx" | "dotx"))
                    })
                    .collect();
                self.catalog.sort();
            }
            if let Some(path) = crate::gpo_storage::resolve_template(root, &p.template) {
                self.catalog.retain(|x| x != &path);
                self.catalog.insert(0, path);
            }
        }
    }
    fn document_mark(app: &WordApp) -> (u64, u64) {
        (app.session.document_id(), app.session.rev())
    }
    fn document_name(app: &WordApp) -> String {
        app.session
            .path
            .as_ref()
            .and_then(|path| path.file_name())
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| "เอกสารที่ยังไม่มีชื่อไฟล์".into())
    }
    fn refresh_fields(&mut self, app: &mut WordApp) {
        let mark = Self::document_mark(app);
        if self.field_doc != Some(mark.0) {
            if let Some(previous) = self.field_doc {
                self.field_drafts.insert(previous, std::mem::take(&mut self.fields));
            }
            self.fields = self.field_drafts.remove(&mark.0).unwrap_or_default();
            self.field_doc = Some(mark.0);
            self.field_mark = None;
            self.active_fields.clear();
        }
        match app.run("gpo.fields", json!({})) {
            Ok(value) => match serde_json::from_value::<Vec<gpo::Finding>>(value["findings"].clone()) {
                Ok(findings) => {
                    let mut names = std::collections::BTreeSet::new();
                    self.manual_fields = 0;
                    for finding in findings {
                        if finding.code == "template-field" {
                            names.insert(finding.suggested);
                        } else {
                            self.manual_fields += 1;
                        }
                    }
                    self.active_fields = names.into_iter().collect();
                    for name in &self.active_fields {
                        self.fields.entry(name.clone()).or_default();
                    }
                    self.field_mark = Some(mark);
                }
                Err(error) => {
                    self.field_mark = None;
                    self.message = format!("อ่านรายการช่องข้อมูลไม่ได้: {error}");
                }
            },
            Err(error) => {
                self.field_mark = None;
                self.message = error;
            }
        }
    }
    fn run_check(&mut self, app: &mut WordApp) {
        let (Some(profile), Some(pack)) = (self.current(), &self.pack) else {
            self.message = "เลือกชนิดเอกสารก่อนตรวจ".into();
            return;
        };
        let glossary = pack.glossary.clone();
        match app.run("gpo.check", json!({"profile":profile,"glossary":glossary})) {
            Ok(value) => match serde_json::from_value::<Vec<gpo::Finding>>(value["findings"].clone()) {
                Ok(findings) => {
                    self.findings = findings;
                    self.checked_mark = Some(Self::document_mark(app));
                    self.checked_name = Self::document_name(app);
                    self.message = format!("พบ {} รายการ — ยังไม่ได้ตรวจการมีอยู่จริงในทะเบียนเอกสาร", self.findings.len());
                }
                Err(error) => {
                    self.checked_mark = None;
                    self.message = format!("อ่านผลตรวจไม่ได้: {error}");
                }
            },
            Err(error) => {
                self.checked_mark = None;
                self.message = error;
            }
        }
    }
    fn result(&mut self, result: Result<Value, String>) {
        self.message = match result {
            Ok(v) => {
                if v.get("pending").is_some() {
                    "รอเลือกหรือยืนยันในหน้าต่างตัวแก้ไข".into()
                } else if v.get("saved").and_then(Value::as_bool) == Some(true) {
                    format!("บันทึกแล้ว: {}", v.get("path").and_then(Value::as_str).unwrap_or(""))
                } else if let Some(written) = v.get("written").and_then(Value::as_array) {
                    if v.get("complete").and_then(Value::as_bool) == Some(true) {
                        format!("สร้าง DOCX แล้ว {} ไฟล์", written.len())
                    } else {
                        format!("สร้างได้ {} ไฟล์ก่อนหยุด: {}", written.len(), v.get("error").and_then(Value::as_str).unwrap_or("เกิดข้อผิดพลาด"))
                    }
                } else if let Some(count) = v.get("replacements").and_then(Value::as_u64) {
                    format!("เติมข้อมูลแล้ว {count} ตำแหน่ง")
                } else if v.get("appended").and_then(Value::as_bool) == Some(true) {
                    "เพิ่มแถวประวัติการแก้ไขแล้ว".into()
                } else if v.get("sourceUnchanged").and_then(Value::as_bool) == Some(true) {
                    format!("ส่งออก PDF แล้ว: {}", v.get("path").and_then(Value::as_str).unwrap_or(""))
                } else if v.get("styles").is_some() {
                    "เพิ่มสไตล์ Head 01–09 แล้ว".into()
                } else {
                    "ดำเนินการสำเร็จ".into()
                }
            }
            Err(e) => e,
        };
    }
    fn save(&mut self) {
        let result = (|| -> Result<(), String> {
            let dir = config_dir()?;
            std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
            if let Some(pack) = &self.pack {
                let text = serde_json::to_string_pretty(pack).map_err(|e| e.to_string())?;
                Pack::parse(&text)?;
                atomic_write(&dir.join("pack.json"), text.as_bytes())?;
            }
            let executable = std::env::current_exe().unwrap_or_default();
            let root = crate::gpo_storage::store_root(&executable, Path::new(&self.root));
            let text = json!({"root":root,"apiUrl":self.api_url,"model":self.model}).to_string();
            atomic_write(&dir.join("settings.json"), text.as_bytes())?;
            Ok(())
        })();
        self.message = result.map(|()| "บันทึกการตั้งค่าแล้ว".to_string()).unwrap_or_else(|e| e);
    }
    pub fn poll(&mut self, ctx: &egui::Context) {
        if let Some((path, receiver)) = &self.template_pending {
            match receiver.try_recv() {
                Ok(result) => {
                    self.template_preview = Some((path.clone(), result));
                    self.template_pending = None;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.message = "ไม่สามารถอ่านตัวอย่างเทมเพลตได้".into();
                    self.template_pending = None;
                }
                Err(mpsc::TryRecvError::Empty) => ctx.request_repaint_after(Duration::from_millis(150)),
            }
        }
        if let Some(rx) = &self.pending {
            match rx.try_recv() {
                Ok(result) => {
                    self.pending = None;
                    match result {
                        Ok(text) => {
                            self.answer = text;
                            self.message = "ได้รับคำตอบแล้ว ตรวจทานก่อนแทรก".into();
                        }
                        Err(e) => self.message = e,
                    }
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.pending = None;
                    self.message = "การเชื่อมต่อสิ้นสุดก่อนมีคำตอบ".into();
                }
                Err(mpsc::TryRecvError::Empty) => ctx.request_repaint_after(Duration::from_millis(150)),
            }
        }
    }
    fn start_template_preview(&mut self) {
        let (Some(path), Some(profile)) = (self.catalog.get(self.selected_template).cloned(), self.current()) else {
            self.message = "เลือกเทมเพลตก่อนดูตัวอย่าง".into();
            return;
        };
        let (sender, receiver) = mpsc::channel();
        self.template_preview = None;
        self.template_pending = Some((path.clone(), receiver));
        std::thread::spawn(move || {
            let result = (|| -> Result<String, String> {
                let mut document = if path.to_string_lossy().to_ascii_lowercase().ends_with(".gpo.json") {
                    let bytes = wordcraft_engine::io::read_file_capped(&path.to_string_lossy(), 2_000_000, "Template")?;
                    gpo::memo_from_blueprint(&bytes)?
                } else {
                    wordcraft_engine::io::open_path(&path)?
                };
                gpo::prepare_template(&mut document, &profile)?;
                let text = gpo::document_text(&document);
                let mut preview: String = text.chars().take(1200).collect();
                if text.chars().count() > 1200 {
                    preview.push_str("\n…");
                }
                if preview.trim().is_empty() {
                    preview = "ไม่มีข้อความตัวอย่างในเทมเพลตนี้".into();
                }
                Ok(preview)
            })();
            let _ = sender.send(result);
        });
    }
    fn show_content(&mut self, ui: &mut egui::Ui, app: &mut WordApp) {
        egui::ScrollArea::vertical().scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible).show(ui, |ui| {
            let previous = self.profile;
            if let Some(pack) = &self.pack {
                egui::ComboBox::from_id_salt("gpo-profile")
                    .selected_text(pack.profiles.get(self.profile).map(|p| p.name.as_str()).unwrap_or("เลือกชนิดเอกสาร"))
                    .show_ui(ui, |ui| {
                        for (i, p) in pack.profiles.iter().enumerate() {
                            ui.selectable_value(&mut self.profile, i, &p.name);
                        }
                    });
            } else {
                ui.label("อ่านชุดกฎเริ่มต้นไม่ได้");
                return;
            }
            if previous != self.profile {
                self.rescan();
                self.findings.clear();
                self.checked_mark = None;
                self.consent = false;
            }
            match self.tab {
                0 => self.documents(ui, app),
                1 => self.review(ui, app),
                2 => self.ai(ui, app),
                3 => self.templates(ui, app),
                4 => self.tools(ui, app),
                _ => self.settings(ui),
            }
            if !self.message.is_empty() {
                ui.separator();
                ui.label(&self.message);
            }
        });
    }
    fn documents(&mut self, ui: &mut egui::Ui, app: &mut WordApp) {
        if app.has_document() && self.field_doc != Some(app.session.document_id()) {
            self.refresh_fields(app);
        }
        ui.label("สร้างจากสำเนาเทมเพลต ต้นฉบับจะไม่เป็นปลายทางบันทึก");
        ui.label("เลือกชนิดเอกสารด้านบน แล้วค้นหาหรือเลือกเทมเพลต");
        ui.add(egui::TextEdit::singleline(&mut self.template_query).hint_text("ค้นหาชื่อเทมเพลต"));
        let query = self.template_query.to_lowercase();
        let choices: Vec<(usize, String)> = self
            .catalog
            .iter()
            .enumerate()
            .filter_map(|(index, path)| {
                let name = path.file_name()?.to_string_lossy().into_owned();
                name.to_lowercase().contains(&query).then_some((index, name))
            })
            .collect();
        if choices.is_empty() {
            ui.label("ไม่พบเทมเพลตในชนิดนี้ ลองค้นหาใหม่หรือตรวจโฟลเดอร์");
        }
        let before = self.selected_template;
        for (index, name) in choices.iter().take(100) {
            ui.selectable_value(&mut self.selected_template, *index, name.as_str());
        }
        if choices.len() > 100 {
            ui.label(format!("แสดง 100 จาก {} รายการ — พิมพ์คำค้นเพื่อจำกัดผล", choices.len()));
        }
        if before != self.selected_template {
            self.template_preview = None;
            self.template_pending = None;
        }
        let selected_visible = choices.iter().any(|(index, _)| *index == self.selected_template);
        if let Some(path) = self.catalog.get(self.selected_template).filter(|_| selected_visible) {
            let source = path
                .strip_prefix(Path::new(&self.root))
                .map(|relative| format!("docsStorage/{}", relative.display()))
                .unwrap_or_else(|_| "ไฟล์ที่เลือกนอกชุด docsStorage".into());
            ui.label(format!("แหล่ง: {source}"));
        }
        let showing_preview = self.template_preview.as_ref().is_some_and(|(path, _)| self.catalog.get(self.selected_template) == Some(path));
        let preview_label = if showing_preview {
            "ซ่อนตัวอย่างข้อความ"
        } else {
            "ดูตัวอย่างข้อความ"
        };
        if ui.add_enabled(self.template_pending.is_none() && selected_visible, egui::Button::new(preview_label)).clicked() {
            if showing_preview {
                self.template_preview = None;
            } else {
                self.start_template_preview();
            }
        }
        if self.template_pending.is_some() {
            ui.spinner();
            ui.label("กำลังอ่านตัวอย่าง…");
        }
        if let Some((path, result)) = &self.template_preview
            && self.catalog.get(self.selected_template) == Some(path)
        {
            ui.label("ตัวอย่างข้อความจากสำเนาเทมเพลต (ไม่ใช่ภาพจัดหน้า)");
            match result {
                Ok(text) => {
                    ui.label(text);
                }
                Err(error) => {
                    ui.label(format!("ดูตัวอย่างไม่ได้: {error}"));
                }
            }
        }
        if ui.add_enabled(selected_visible, egui::Button::new("สร้างเอกสารจากเทมเพลต")).clicked() {
            if let (Some(path), Some(p)) = (self.catalog.get(self.selected_template), self.current()) {
                let result = app.run("file.new", json!({"template":"gpo","path":path,"profile":p}));
                if result.as_ref().is_ok_and(|value| value.get("pending").is_none()) {
                    self.refresh_fields(app);
                }
                self.result(result);
            } else {
                self.message = "เลือกเทมเพลตในแท็บเทมเพลตก่อน".into();
            }
        }
        ui.horizontal_wrapped(|ui| {
            if ui.button("เพิ่มสไตล์ Head 01–09").clicked()
                && let Some(p) = self.current()
            {
                self.result(app.run("gpo.styles", json!({"profile":p})));
            }
            for n in 1..=3 {
                if ui.button(format!("Head {n:02}")).clicked() {
                    self.result(app.run("para.style", json!({"style":format!("Head {n:02}")})));
                }
            }
            if ui.button("เนื้อหาปกติ").clicked() {
                self.result(app.run("para.style", json!({"style":"Normal"})));
            }
        });
        ui.label("Head 01 → 1.    Head 02 → 1.1    Head 03 → 1.1.1");
        egui::CollapsingHeader::new("ข้อมูลที่ยังไม่ครบในเอกสาร").default_open(true).show(ui, |ui| {
            let current = self.field_mark == Some(Self::document_mark(app));
            if !current {
                ui.label("เอกสารเปลี่ยนแล้ว ค้นหาช่องอีกครั้งก่อนเติมข้อมูล");
            }
            if ui.button("ค้นหาช่องข้อมูลอีกครั้ง").clicked() {
                self.refresh_fields(app);
            }
            if self.active_fields.is_empty() && self.manual_fields == 0 && current {
                ui.label("ไม่พบช่อง {{field}} ที่ค้างอยู่");
            }
            if self.manual_fields > 0 {
                ui.label(format!("มี {} ช่องในฟิลด์หรือส่วนที่ติดตามการแก้ไข ให้ตรวจและแก้ด้วยตนเอง", self.manual_fields));
            }
            for key in &self.active_fields {
                ui.label(key);
                if let Some(value) = self.fields.get_mut(key) {
                    ui.text_edit_singleline(value);
                }
            }
            if !self.active_fields.is_empty() {
                ui.label("เว้นว่างไว้ได้ ช่องนั้นจะยังอยู่ในเอกสาร");
            }
            if ui.add_enabled(current && !self.active_fields.is_empty(), egui::Button::new("เติมข้อมูลที่กรอก")).clicked()
            {
                let values: BTreeMap<_, _> = self
                    .active_fields
                    .iter()
                    .filter_map(|key| self.fields.get(key).filter(|value| !value.trim().is_empty()).map(|value| (key.clone(), value.clone())))
                    .collect();
                let result = app.run("gpo.fill", json!({"values":values}));
                if result.is_ok() {
                    self.refresh_fields(app);
                }
                self.result(result);
            }
        });
        if ui.add_enabled(app.has_document(), egui::Button::new("บันทึกเป็นไฟล์ใหม่…")).clicked()
            && let Some(path) = rfd::FileDialog::new().add_filter("Word document", &["docx"]).save_file()
        {
            self.result(app.run("file.saveAs", json!({"path":path,"noOverwrite":true})));
        }
        ui.separator();
        ui.checkbox(&mut self.watermark_on, "ลายน้ำ");
        ui.text_edit_singleline(&mut self.watermark);
        ui.horizontal(|ui| {
            if ui.button("ใช้ลายน้ำกับเอกสาร").clicked() {
                self.result(app.run(
                    "design.watermark",
                    if self.watermark_on {
                        json!({"text":self.watermark,"font":wordcraft_doc::styles::THAI_FONT,"diagonal":true})
                    } else {
                        json!({"remove":true})
                    },
                ));
            }
            if ui.button("ส่งออก PDF…").clicked()
                && let Some(path) = rfd::FileDialog::new().add_filter("PDF", &["pdf"]).save_file()
            {
                let result = app
                    .run("gpo.exportPdf", json!({"path":path,"watermark":if self.watermark_on{Some(&self.watermark)}else{None},"noOverwrite":true}));
                if result.is_ok() {
                    self.exported_mark = Some(Self::document_mark(app));
                }
                self.result(result);
            }
        });
        ui.separator();
        let request_batch = std::mem::take(&mut self.request_batch);
        let batch = egui::CollapsingHeader::new("สร้าง FM / LG หลายชุด").open(if request_batch { Some(true) } else { None }).show(ui, |ui| {
            ui.label("เลือก CSV UTF-8 และโฟลเดอร์ แล้วจับคู่ชื่อช่องก่อนตรวจตัวอย่าง");
            ui.label("ไฟล์ CSV");
            ui.text_edit_singleline(&mut self.merge_source);
            if ui.button("เลือก CSV…").clicked()
                && let Some(p) = rfd::FileDialog::new().add_filter("CSV", &["csv"]).pick_file()
            {
                self.merge_source = p.to_string_lossy().into();
            }
            ui.label("โฟลเดอร์ผลลัพธ์");
            ui.text_edit_singleline(&mut self.merge_output);
            if ui.button("เลือกโฟลเดอร์…").clicked()
                && let Some(p) = rfd::FileDialog::new().pick_folder()
            {
                self.merge_output = p.to_string_lossy().into();
            }
            if let Some(preview) = &self.merge_preview {
                let headers = preview["headers"].as_array().cloned().unwrap_or_default();
                let fields = preview["fields"].as_array().cloned().unwrap_or_default();
                ui.label("คอลัมน์สำหรับชื่อไฟล์");
                egui::ComboBox::from_id_salt("merge-name").selected_text(&self.merge_name_field).show_ui(ui, |ui| {
                    for header in &headers {
                        if let Some(name) = header.as_str() {
                            ui.selectable_value(&mut self.merge_name_field, name.to_owned(), name);
                        }
                    }
                });
                for field in &fields {
                    if let Some(field) = field.as_str() {
                        let mapped = self.merge_mapping.entry(field.into()).or_insert_with(|| field.into());
                        ui.label(format!("{{{{{field}}}}}"));
                        egui::ComboBox::from_id_salt(("merge-map", field)).selected_text(mapped.as_str()).show_ui(ui, |ui| {
                            for header in &headers {
                                if let Some(name) = header.as_str() {
                                    ui.selectable_value(mapped, name.to_owned(), name);
                                }
                            }
                        });
                    }
                }
                ui.label(format!("{} แถว · ตัวอย่างไม่เกิน 3 แถว", preview["records"].as_u64().unwrap_or(0)));
                if let Some(rows) = preview["sample"].as_array() {
                    for row in rows {
                        if let Some(values) = row.as_array() {
                            ui.label(values.iter().filter_map(Value::as_str).collect::<Vec<_>>().join(" | "));
                        }
                    }
                }
                for name in preview["outputNames"].as_array().into_iter().flatten().filter_map(Value::as_str) {
                    ui.label(name);
                }
                if let Some(error) = preview["error"].as_str().filter(|s| !s.is_empty()) {
                    ui.label("ยังสร้างไม่ได้ โปรดตรวจคอลัมน์ ข้อมูล และชื่อไฟล์");
                    ui.label(error);
                }
            }
            let args = json!({"csv":self.merge_source,"directory":self.merge_output,"nameField":self.merge_name_field,"mapping":self.merge_mapping});
            let signature = format!("{}:{:?}", args, Self::document_mark(app));
            if ui.button("ตรวจตัวอย่างและชื่อไฟล์").clicked() {
                match app.run("gpo.mergePreview", args.clone()) {
                    Ok(preview) => {
                        self.merge_preview = Some(preview);
                        self.merge_signature = signature.clone();
                        self.message.clear();
                    }
                    Err(error) => {
                        self.merge_preview = None;
                        self.merge_signature.clear();
                        self.message = format!("ตรวจไฟล์ CSV และโฟลเดอร์ผลลัพธ์: {error}");
                    }
                }
            }
            let ready =
                self.merge_signature == signature && self.merge_preview.as_ref().is_some_and(|preview| preview["ready"].as_bool() == Some(true));
            if self.merge_preview.is_some() && self.merge_signature != signature {
                ui.label("ข้อมูลเปลี่ยนแล้ว กดตรวจตัวอย่างอีกครั้ง");
            }
            if ui.add_enabled(ready, egui::Button::new("สร้าง DOCX แยกตามแถว")).clicked() {
                self.merge_signature.clear();
                let mut merge_args = args;
                merge_args["csvDigest"] = self.merge_preview.as_ref().and_then(|preview| preview.get("csvDigest")).cloned().unwrap_or(Value::Null);
                self.result(app.run("gpo.merge", merge_args));
            }
            ui.label("ไฟล์ที่มีอยู่จะไม่ถูกเขียนทับ หากหยุดกลางทางจะแจ้งจำนวนที่สร้างแล้ว");
        });
        if request_batch {
            batch.header_response.scroll_to_me(Some(egui::Align::TOP));
        }
    }
    fn review(&mut self, ui: &mut egui::Ui, app: &mut WordApp) {
        if ui.add_enabled(app.has_document(), egui::Button::new("ตรวจหัวข้อ คำศัพท์ และข้อมูลที่ยังไม่ครบ")).clicked()
        {
            self.run_check(app);
        }
        let mark = Self::document_mark(app);
        let fresh = self.checked_mark == Some(mark) && app.has_document();
        if self.checked_mark.is_some() {
            ui.label(format!("ผลตรวจ: {}", self.checked_name));
            if !fresh {
                ui.label("เอกสารเปลี่ยนแล้ว ผลตรวจเดิมอาจไม่ตรง กดตรวจอีกครั้ง");
            } else if self.findings.is_empty() {
                ui.label("ไม่พบรายการจากกฎที่เลือก — ยังไม่ได้ตรวจทะเบียนหรืออนุมัติเอกสาร");
            }
        }
        let mut jump = None;
        for (category, title) in
            [("template", "ข้อมูลยังไม่ครบ"), ("missing-heading", "หัวข้อที่ไม่พบ"), ("terminology", "คำศัพท์"), ("reference-format", "รูปแบบรหัสอ้างอิง")]
        {
            let matches =
                |finding: &&gpo::Finding| if category == "template" { finding.code.starts_with("template-field") } else { finding.code == category };
            let count = self.findings.iter().filter(matches).count();
            if count == 0 {
                continue;
            }
            egui::CollapsingHeader::new(format!("{title} ({count})")).default_open(true).show(ui, |ui| {
                for finding in self.findings.iter().filter(matches) {
                    ui.label(&finding.message);
                    if !finding.found.is_empty() {
                        if category == "terminology" && !finding.suggested.is_empty() {
                            ui.label(format!("{} → {}", finding.found, finding.suggested));
                        } else {
                            ui.label(&finding.found);
                        }
                    } else if !finding.suggested.is_empty() {
                        ui.label(&finding.suggested);
                    }
                    if let Some(at) = &finding.location
                        && ui.add_enabled(fresh, egui::Button::new("ไปยังข้อความ")).clicked()
                    {
                        jump = Some(at.clone());
                    }
                    ui.separator();
                }
            });
        }
        if let Some(at) = jump {
            if let Err(error) = app.run("select.range", json!({"anchor":at,"focus":at})) {
                self.message = error;
            } else {
                app.canvas.scroll_to_caret = true;
                app.canvas.want_focus = true;
            }
        }
        ui.separator();
        if ui.button("เลือกฉบับเดิมเพื่อเทียบกับฉบับที่เปิด…").clicked()
            && let Some(path) = rfd::FileDialog::new().add_filter("Word", &["doc", "docx"]).pick_file()
        {
            match app.run("gpo.compare", json!({"original":path})) {
                Ok(value) => match serde_json::from_value::<Vec<gpo::Change>>(value["changes"].clone()) {
                    Ok(changes) => {
                        self.changes = changes;
                        self.comparison_mark = Some(Self::document_mark(app));
                        self.comparison_source = path.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
                        self.message = format!("พบความต่าง {} ส่วน", self.changes.len());
                    }
                    Err(error) => {
                        self.comparison_mark = None;
                        self.message = format!("อ่านผลเปรียบเทียบไม่ได้: {error}");
                    }
                },
                Err(error) => {
                    self.comparison_mark = None;
                    self.message = error;
                }
            }
        }
        let comparison_fresh = self.comparison_mark == Some(mark) && app.has_document();
        if self.comparison_mark.is_some() {
            ui.label(format!("เทียบ {} กับ {}", self.comparison_source, Self::document_name(app)));
            if !comparison_fresh {
                ui.label("เอกสารเปลี่ยนแล้ว ให้เปรียบเทียบอีกครั้งก่อนใช้ผลเดิม");
            } else if self.changes.is_empty() {
                ui.label("ไม่พบความต่างในส่วนที่เครื่องมือนี้ตรวจ");
            }
        }
        for change in &self.changes {
            let title = match change.section.as_str() {
                "styles" => "สไตล์เอกสาร",
                "numbering" => "เลขหัวข้อ",
                "page-setup" => "การตั้งค่าหน้ากระดาษ",
                _ => &change.section,
            };
            egui::CollapsingHeader::new(title).show(ui, |ui| {
                if matches!(change.section.as_str(), "styles" | "numbering" | "page-setup") {
                    ui.label("พบการเปลี่ยนแปลงรูปแบบในส่วนนี้ ควรตรวจหน้าจริงเพิ่มเติม");
                    egui::CollapsingHeader::new("รายละเอียดทางเทคนิค").show(ui, |ui| {
                        ui.strong("ฉบับเดิม");
                        ui.label(&change.before);
                        ui.strong("ฉบับปัจจุบัน");
                        ui.label(&change.after);
                    });
                } else {
                    ui.strong("ฉบับเดิม");
                    ui.label(&change.before);
                    ui.separator();
                    ui.strong("ฉบับปัจจุบัน");
                    ui.label(&change.after);
                }
            });
        }
        if ui.add_enabled(comparison_fresh && !self.changes.is_empty(), egui::Button::new("เตรียมให้ AI สรุป Revision History")).clicked()
        {
            self.context = serde_json::to_string_pretty(&self.changes).unwrap_or_default();
            self.task = "สรุปการเปลี่ยนแปลงสำหรับตารางประวัติการแก้ไข ระบุเฉพาะสิ่งที่เปลี่ยนจริง แยกการแก้เนื้อหากับรูปแบบ หากบอกเหตุผลไม่ได้ห้ามเดา".into();
            self.consent = false;
            self.consent_receipt = None;
            self.tab = 2;
        }
        ui.separator();
        ui.label("เพิ่มแถวประวัติการแก้ไขหลังตรวจทานคำสรุปแล้ว");
        let doc_id = app.session.document_id();
        if self.revision_doc.is_some_and(|id| id != doc_id) {
            ui.label("ข้อมูลประวัตินี้เริ่มจากอีกเอกสาร โปรดตรวจให้ตรงก่อนใช้");
            if ui.button("ใช้ข้อมูลนี้กับเอกสารปัจจุบัน").clicked() {
                self.revision_doc = Some(doc_id);
            }
        }
        let mut edited = false;
        ui.label("Revision");
        edited |= ui.text_edit_singleline(&mut self.revision).changed();
        ui.label("วันที่บังคับใช้");
        edited |= ui.text_edit_singleline(&mut self.revision_date).changed();
        ui.label("ผู้จัดทำ (ถ้าตารางมีคอลัมน์นี้)");
        edited |= ui.text_edit_singleline(&mut self.revision_author).changed();
        ui.label("สรุปการแก้ไขที่ตรวจแล้ว");
        edited |= ui.text_edit_multiline(&mut self.revision_summary).changed();
        if edited && self.revision_doc.is_none() {
            self.revision_doc = Some(doc_id);
        }
        if ui.add_enabled(app.has_document() && self.revision_doc == Some(doc_id), egui::Button::new("เพิ่มแถวจากข้อความที่ตรวจแล้ว")).clicked()
        {
            self.result(app.run(
                "gpo.revision",
                json!({"revision":self.revision,"date":self.revision_date,"summary":self.revision_summary,"author":self.revision_author}),
            ));
        }
    }
    fn ai(&mut self, ui: &mut egui::Ui, app: &mut WordApp) {
        ui.horizontal_wrapped(|ui| {
            ui.label(format!("ผู้ให้บริการ: {} · {}", self.api_url, self.model));
            if ui.button("ตั้งค่าผู้ให้บริการและ Key").clicked() {
                self.tab = 5;
            }
        });
        ui.label("งานที่ต้องการให้ช่วย");
        ui.text_edit_multiline(&mut self.task);
        ui.horizontal_wrapped(|ui| {
            if ui.button("ใช้ข้อความจากเอกสาร").clicked() {
                self.context = gpo::document_text(&app.session.doc);
            }
            if ui.button("ใช้เฉพาะข้อความที่เลือก").clicked() {
                let (a, b) = app.session.sel.ordered();
                self.context = app.session.doc.copy_range(&a, &b).plain_text();
            }
        });
        ui.label("เนื้อหาอ้างอิง — แก้ไขหรือลบข้อมูลได้ก่อนส่ง");
        ui.add(egui::TextEdit::multiline(&mut self.context).desired_rows(7));
        let request = self.request_preview();
        if let Ok((endpoint, body)) = &request {
            if self.consent
                && !self.consent_receipt.as_ref().is_some_and(|(saved_endpoint, saved_body)| saved_endpoint == endpoint && saved_body == body)
            {
                self.consent = false;
                self.consent_receipt = None;
            }
            egui::CollapsingHeader::new("ดูข้อมูลทั้งหมดที่จะส่ง").show(ui, |ui| {
                ui.label(format!("POST {endpoint}"));
                ui.label("ใช้ API Key ที่ตั้งไว้ในส่วน Authorization โดยไม่แสดง Key ที่นี่");
                if let Ok(payload) = serde_json::from_str::<Value>(body) {
                    ui.label(format!(
                        "Model: {} · temperature: {} · max tokens: {} · stream: {}",
                        payload["model"], payload["temperature"], payload["max_tokens"], payload["stream"]
                    ));
                    for (title, pointer) in [("คำสั่งระบบที่ส่งไปด้วย", "/messages/0/content"), ("คำขอและเนื้อหาที่ส่ง", "/messages/1/content")]
                    {
                        ui.strong(title);
                        egui::ScrollArea::vertical().id_salt(pointer).max_height(150.0).show(ui, |ui| {
                            ui.label(payload.pointer(pointer).and_then(Value::as_str).unwrap_or(""));
                        });
                    }
                }
                egui::CollapsingHeader::new("รายละเอียด JSON ของคำขอ").show(ui, |ui| {
                    let mut preview = body.as_str();
                    ui.add(egui::TextEdit::multiline(&mut preview).desired_rows(6).interactive(false));
                });
            });
            if ui.checkbox(&mut self.consent, "ตรวจข้อมูลและยินยอมส่งคำขอนี้แล้ว").changed()
            {
                self.consent_receipt = self.consent.then(|| (endpoint.clone(), body.clone()));
            }
        } else if let Err(error) = request {
            self.consent = false;
            self.consent_receipt = None;
            ui.label(error);
        }
        if ui.add_enabled(self.pending.is_none() && self.consent, egui::Button::new("ให้ AI ช่วยเขียน")).clicked() {
            self.start_ai(false);
        }
        if self.pending.is_some() {
            ui.spinner();
            ui.label("กำลังรอคำตอบ…");
        }
        ui.label("คำตอบ AI ที่แก้ไขได้");
        ui.add(egui::TextEdit::multiline(&mut self.answer).desired_rows(8));
        ui.horizontal_wrapped(|ui| {
            if ui.button("แทรกข้อความที่ตรวจแล้ว ณ ตำแหน่งเลือก").clicked() {
                self.result(app.run("text.insert", json!({"text":self.answer})));
            }
            if ui.button("ใช้เป็นสรุป Revision").clicked() {
                self.revision_summary = self.answer.clone();
                self.revision_doc = Some(app.session.document_id());
                self.tab = 1;
            }
        });
    }
    fn request_preview(&self) -> Result<(String, String), String> {
        validate_url(&self.api_url)?;
        let profile = self.current().ok_or("No document profile")?;
        let (system, user) = gpo::prompt(&profile, &self.context, &self.task)?;
        let endpoint = format!("{}/chat/completions", self.api_url.trim_end_matches('/'));
        Ok((endpoint, ai_request_body(&self.model, &system, &user).to_string()))
    }
    fn templates(&mut self, ui: &mut egui::Ui, app: &mut WordApp) {
        ui.label("แก้รูปแบบโดยเปิดสำเนาเทมเพลต แก้หัวกระดาษ แล้ว Save As เป็น DOCX / DOTX รุ่นใหม่");
        if ui.button("เลือกเทมเพลตสำหรับชนิดนี้…").clicked()
            && let Some(path) = rfd::FileDialog::new().add_filter("GPO / Word template", &["doc", "docx", "dotx", "json"]).pick_file()
        {
            if let Some(p) = self.pack.as_mut().and_then(|p| p.profiles.get_mut(self.profile)) {
                p.template = crate::gpo_storage::store_template(Path::new(&self.root), &path);
            }
            self.rescan();
            self.save();
        }
        if let Some(p) = self.pack.as_mut().and_then(|p| p.profiles.get_mut(self.profile)) {
            ui.label("ชื่อชนิดเอกสาร");
            ui.text_edit_singleline(&mut p.name);
            ui.label("รุ่นเทมเพลต");
            ui.text_edit_singleline(&mut p.template_version);
            ui.label("ฟอนต์ Head 01–09");
            ui.text_edit_singleline(&mut p.font);
            ui.horizontal(|ui| {
                ui.label("ขนาดตัวอักษร (pt)");
                ui.add(egui::DragValue::new(&mut p.font_size).range(10.0..=24.0).speed(0.5));
            });
            ui.checkbox(&mut p.memo_header_only, "ใช้เฉพาะหัวบันทึกข้อความ และเว้นเนื้อหาใหม่");
            if self.active_profile_editor != Some(self.profile) {
                self.heading_lines = p.required_headings.join("\n");
                self.active_profile_editor = Some(self.profile);
            }
            ui.label("หัวข้อที่ต้องมี — หนึ่งหัวข้อต่อบรรทัด");
            if ui.text_edit_multiline(&mut self.heading_lines).changed() {
                p.required_headings = self.heading_lines.lines().map(str::trim).filter(|s| !s.is_empty()).map(str::to_string).collect();
            }
            ui.label("แนวทางการเขียนเพิ่มเติม");
            ui.text_edit_multiline(&mut p.skill);
        }
        if ui.button("บันทึกรุ่นและกฎเทมเพลต").clicked() {
            self.save();
        }
        ui.separator();
        ui.label("ผูกช่องข้อมูล: เลือกข้อความตัวอย่างในเอกสาร แล้วกดชื่อช่องเพื่อแทนด้วย {{field}} จากนั้นบันทึกเป็นเทมเพลตรุ่นใหม่");
        let mut binding_fields: std::collections::BTreeSet<String> =
            ["documentNumber", "revision", "date", "title", "department", "phone", "recipient", "signatory", "position", "machineCode"]
                .into_iter()
                .map(str::to_string)
                .collect();
        binding_fields.extend(self.active_fields.iter().cloned());
        for key in binding_fields {
            if ui.small_button(format!("ผูก {key}")).clicked() {
                self.result(app.run("text.insert", json!({"text":format!("{{{{{key}}}}}")})));
            }
        }
        ui.label("Logbook ใช้หน้าปกแล้วเพิ่มตารางด้วย Insert › Table ได้ตามงาน");
    }
    fn tools(&mut self, ui: &mut egui::Ui, app: &mut WordApp) {
        egui::ComboBox::from_id_salt("gpo-thai-tool")
            .selected_text(["เลขไทย", "เลขอารบิก", "จำนวนเงินเป็นคำอ่าน", "วันที่ พ.ศ."].get(self.utility_mode).copied().unwrap_or(""))
            .show_ui(ui, |ui| {
                for (i, name) in ["เลขไทย", "เลขอารบิก", "จำนวนเงินเป็นคำอ่าน", "วันที่ พ.ศ."].iter().enumerate()
                {
                    ui.selectable_value(&mut self.utility_mode, i, *name);
                }
            });
        ui.label("วันที่ใช้ YYYY-MM-DD; จำนวนเงินไม่ใส่ comma");
        ui.checkbox(&mut self.buddhist_input, "ปีวันที่ที่กรอกเป็น พ.ศ. อยู่แล้ว");
        ui.label("ข้อความหรือค่าที่ต้องการแปลง");
        ui.text_edit_multiline(&mut self.utility);
        if ui.button("แปลง").clicked() {
            let mode = ["thaiDigits", "arabicDigits", "baht", "date"].get(self.utility_mode).copied().unwrap_or("date");
            match app.run("gpo.thai", json!({"mode":mode,"text":self.utility,"buddhistInput":self.buddhist_input})) {
                Ok(v) => self.utility_output = v["text"].as_str().unwrap_or("").into(),
                Err(e) => self.message = e,
            }
        }
        ui.label("ผลลัพธ์");
        ui.text_edit_multiline(&mut self.utility_output);
        if ui.button("แทรกผลลัพธ์").clicked() {
            self.result(app.run("text.insert", json!({"text":self.utility_output})));
        }
    }
    fn settings(&mut self, ui: &mut egui::Ui) {
        ui.heading("ที่เก็บเทมเพลต");
        ui.label("โฟลเดอร์ docsStorage");
        ui.text_edit_singleline(&mut self.root);
        if ui.button("เลือกโฟลเดอร์…").clicked()
            && let Some(p) = rfd::FileDialog::new().pick_folder()
        {
            self.root = p.to_string_lossy().into();
            self.rescan();
        }
        ui.separator();
        ui.heading("AI และ API Key");
        ui.label("API Base URL");
        if ui.text_edit_singleline(&mut self.api_url).changed() {
            self.api_key = Password::default();
        }
        ui.label("Model");
        ui.text_edit_singleline(&mut self.model);
        ui.label("API Key — เก็บด้วย Windows DPAPI ของบัญชีผู้ใช้");
        ui.add(egui::TextEdit::singleline(self.api_key.as_mut_string()).password(true));
        if ui.button("บันทึก Key").clicked() {
            match save_key(&self.api_url, self.api_key.as_str()) {
                Ok(()) => {
                    self.message = "บันทึก Key แบบเข้ารหัสแล้ว".into();
                    self.api_key = Password::default();
                }
                Err(e) => self.message = format!("{e} — ยังใช้ Key ในเซสชันนี้ได้โดยไม่บันทึก"),
            }
        }
        if ui.button("ลบ Key ที่บันทึก").clicked() {
            self.message = delete_key().map(|()| "ลบ Key แล้ว".into()).unwrap_or_else(|e| e);
            self.api_key = Password::default();
        }
        if ui.add_enabled(self.pending.is_none(), egui::Button::new("ทดสอบ API ด้วยข้อความทดสอบ")).clicked()
        {
            self.start_ai(true);
        }
        if ui.button("บันทึกการตั้งค่า").clicked() {
            self.save();
            self.rescan();
        }
        ui.separator();
        ui.heading("พจนานุกรม");
        ui.label("รายการเริ่มต้นเป็นเพียงคำผิดทั่วไป เพิ่มศัพท์ GMP ของทีมได้ที่นี่");
        if let Some(pack) = &mut self.pack {
            let mut remove = None;
            for (i, rule) in pack.glossary.iter_mut().enumerate() {
                ui.push_id(i, |ui| {
                    let field_width = ((ui.available_width() - 55.0) / 2.0).clamp(60.0, 135.0);
                    ui.horizontal(|ui| {
                        ui.add(egui::TextEdit::singleline(&mut rule.find).desired_width(field_width));
                        ui.label("→");
                        ui.add(egui::TextEdit::singleline(&mut rule.preferred).desired_width(field_width));
                        if ui.small_button("ลบ").clicked() {
                            remove = Some(i);
                        }
                    });
                    ui.text_edit_singleline(&mut rule.note);
                });
            }
            if let Some(i) = remove {
                pack.glossary.remove(i);
            }
            if ui.button("เพิ่มคำศัพท์").clicked() {
                pack.glossary.push(gpo::TermRule { find: String::new(), preferred: String::new(), note: String::new() });
            }
        }
        if ui.button("บันทึกพจนานุกรม").clicked() {
            self.save();
        }
    }
    fn start_ai(&mut self, test: bool) {
        let result = (|| -> Result<_, String> {
            validate_url(&self.api_url)?;
            let profile = self.current().ok_or("No document profile")?;
            let messages = if test {
                ("ตอบเป็นภาษาไทย".into(), "ตอบว่า เชื่อมต่อสำเร็จ".into())
            } else {
                let request = self.request_preview()?;
                if !self.consent || self.consent_receipt.as_ref() != Some(&request) {
                    return Err("ตรวจและยืนยันข้อมูลที่จะส่งอีกครั้ง".into());
                }
                gpo::prompt(&profile, &self.context, &self.task)?
            };
            let key = if self.api_key.is_empty() { load_key(&self.api_url)? } else { Password::new(self.api_key.as_str()) };
            if key.is_empty() {
                return Err("ใส่ API Key ในการตั้งค่าก่อน".into());
            }
            Ok((messages, key))
        })();
        match result {
            Err(e) => self.message = e,
            Ok((messages, key)) => {
                let (url, model) = (self.api_url.clone(), self.model.clone());
                let (tx, rx) = mpsc::channel();
                self.pending = Some(rx);
                std::thread::spawn(move || {
                    let _ = tx.send(complete(&url, &model, key, messages));
                });
            }
        }
    }
}

fn config_dir() -> Result<PathBuf, String> {
    if let Ok(executable) = std::env::current_exe()
        && let Some(directory) = crate::gpo_storage::portable_dir(&executable)
    {
        return Ok(directory.join("data"));
    }
    let base = std::env::var_os("APPDATA")
        .or_else(|| std::env::var_os("XDG_CONFIG_HOME"))
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|p| PathBuf::from(p).join(".config")))
        .ok_or("Cannot locate settings directory")?;
    Ok(base.join("GPOWord"))
}
fn read_bounded(path: &Path, limit: u64) -> Result<String, String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut text = String::new();
    file.take(limit + 1).read_to_string(&mut text).map_err(|e| e.to_string())?;
    if text.len() as u64 > limit {
        return Err("File is too large".into());
    }
    Ok(text)
}
fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
    let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(&temporary).map_err(|e| e.to_string())?;
    file.write_all(bytes).and_then(|()| file.sync_all()).map_err(|e| e.to_string())?;
    drop(file);
    // On Windows rename does not replace. Backup first, restore on failure.
    let backup = path.with_extension("previous");
    if path.exists() {
        if backup.exists() {
            std::fs::remove_file(&backup).map_err(|e| e.to_string())?;
        }
        std::fs::rename(path, &backup).map_err(|e| e.to_string())?;
    }
    if let Err(e) = std::fs::rename(&temporary, path) {
        if backup.exists() {
            let _ = std::fs::rename(&backup, path);
        }
        let _ = std::fs::remove_file(&temporary);
        return Err(e.to_string());
    }
    Ok(())
}
fn validate_url(url: &str) -> Result<(), String> {
    if url.contains(['\r', '\n', '?', '#', '@'])
        || url.len() > 512
        || !(url.starts_with("https://") || url.starts_with("http://127.0.0.1:") || url.starts_with("http://localhost:"))
    {
        return Err("ใช้ HTTPS หรือ HTTP บน localhost เท่านั้น".into());
    }
    Ok(())
}
fn ai_request_body(model: &str, system: &str, user: &str) -> Value {
    json!({"model":model,"messages":[{"role":"system","content":system},{"role":"user","content":user}],"temperature":0.2,"max_tokens":2048,"stream":false})
}
fn complete(url: &str, model: &str, key: Password, (system, user): (String, String)) -> Result<String, String> {
    validate_url(url)?;
    let body = ai_request_body(model, &system, &user).to_string();
    let agent = ureq::AgentBuilder::new().timeout(Duration::from_secs(120)).redirects(0).build();
    let response = agent
        .post(&format!("{}/chat/completions", url.trim_end_matches('/')))
        .set("Authorization", &format!("Bearer {}", key.as_str()))
        .set("Content-Type", "application/json")
        .send_string(&body)
        .map_err(|e| match e {
            ureq::Error::Status(code, _) => format!("API ตอบ HTTP {code} — ตรวจ Key โมเดล และโควตา; ไม่มีการเปลี่ยนไปใช้บริการอื่นอัตโนมัติ"),
            ureq::Error::Transport(_) => "เชื่อมต่อ API ไม่สำเร็จหรือหมดเวลารอ".into(),
        })?;
    let mut text = String::new();
    response.into_reader().take(2_000_001).read_to_string(&mut text).map_err(|_| "อ่านคำตอบ API ไม่สำเร็จ")?;
    if text.len() > 2_000_000 {
        return Err("API response is too large".into());
    }
    let value: Value = serde_json::from_str(&text).map_err(|_| "API ตอบข้อมูลที่ไม่ใช่ JSON")?;
    value
        .pointer("/choices/0/message/content")
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| "API ไม่มีข้อความตอบกลับตามรูปแบบ Chat Completions".into())
}

fn save_key(endpoint: &str, key: &str) -> Result<(), String> {
    validate_url(endpoint)?;
    if key.is_empty() || key.len() > 16_000 {
        return Err("Invalid API Key".into());
    }
    let cipher = dpapi(key, true)?;
    let dir = config_dir()?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    atomic_write(&dir.join("api-key.dpapi"), json!({"endpoint":endpoint,"cipher":cipher}).to_string().as_bytes())
}
fn load_key(endpoint: &str) -> Result<Password, String> {
    let text = read_bounded(&config_dir()?.join("api-key.dpapi"), 100_000).map_err(|_| "ยังไม่มี Key ที่บันทึกไว้")?;
    let value: Value = serde_json::from_str(&text).map_err(|_| "Key storage is invalid")?;
    if value["endpoint"].as_str() != Some(endpoint) {
        return Err("API URL เปลี่ยนแล้ว กรุณาบันทึก Key สำหรับ URL นี้ใหม่".into());
    }
    let plain = dpapi(value["cipher"].as_str().ok_or("Invalid encrypted key")?, false)?;
    Ok(Password::new(&plain))
}
fn delete_key() -> Result<(), String> {
    let dir = config_dir()?;
    for name in ["api-key.dpapi", "api-key.previous"] {
        let path = dir.join(name);
        if path.is_file() {
            std::fs::remove_file(path).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
#[cfg(target_os = "windows")]
fn dpapi(input: &str, encrypt: bool) -> Result<String, String> {
    use std::os::windows::process::CommandExt;
    use std::process::{Command, Stdio};
    let operation = if encrypt {
        "[Convert]::ToBase64String([Security.Cryptography.ProtectedData]::Protect([Text.Encoding]::UTF8.GetBytes($v),$null,[Security.Cryptography.DataProtectionScope]::CurrentUser))"
    } else {
        "[Text.Encoding]::UTF8.GetString([Security.Cryptography.ProtectedData]::Unprotect([Convert]::FromBase64String($v),$null,[Security.Cryptography.DataProtectionScope]::CurrentUser))"
    };
    let script = format!(
        "[Console]::InputEncoding=[Text.UTF8Encoding]::new();[Console]::OutputEncoding=[Text.UTF8Encoding]::new();$ErrorActionPreference='Stop';[void][Reflection.Assembly]::Load('System.Security, Version=4.0.0.0, Culture=neutral, PublicKeyToken=b03f5f7f11d50a3a');$v=[Console]::In.ReadToEnd();{operation}"
    );
    let mut child = Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .creation_flags(0x08000000)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| "Windows DPAPI helper could not start")?;
    let mut pipe = child.stdin.take().ok_or("DPAPI input is unavailable")?;
    pipe.write_all(input.as_bytes()).map_err(|_| "DPAPI input failed")?;
    drop(pipe);
    let output = child.wait_with_output().map_err(|_| "DPAPI helper failed")?;
    if !output.status.success() {
        return Err("Windows could not protect or read this Key for the current user".into());
    }
    String::from_utf8(output.stdout).map(|s| s.trim_end_matches(['\r', '\n']).to_string()).map_err(|_| "DPAPI output is invalid".into())
}
#[cfg(not(target_os = "windows"))]
fn dpapi(_input: &str, _encrypt: bool) -> Result<String, String> {
    Err("Persistent Key storage is available on Windows; other platforms can use a session-only Key".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ai_request_preview_matches_the_local_mock_request() {
        use std::net::TcpListener;
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            let mut request = Vec::new();
            let mut buffer = [0u8; 4096];
            loop {
                let n = stream.read(&mut buffer).unwrap();
                assert!(n > 0);
                request.extend_from_slice(&buffer[..n]);
                if let Some(end) = request.windows(4).position(|w| w == b"\r\n\r\n") {
                    let header = String::from_utf8_lossy(&request[..end]);
                    let length: usize = header
                        .lines()
                        .find_map(|line| line.to_ascii_lowercase().strip_prefix("content-length:").map(str::trim).and_then(|v| v.parse().ok()))
                        .unwrap();
                    if request.len() >= end + 4 + length {
                        break;
                    }
                }
            }
            let response = r#"{"choices":[{"message":{"content":"review me"}}]}"#;
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                response.len(),
                response
            )
            .unwrap();
            String::from_utf8(request).unwrap()
        });
        let expected = ai_request_body("mock-model", "system example", "synthetic document").to_string();
        assert!(!expected.contains("fake-local-test-key"));
        let answer = complete(
            &format!("http://{address}"),
            "mock-model",
            Password::new("fake-local-test-key"),
            ("system example".into(), "synthetic document".into()),
        )
        .unwrap();
        assert_eq!(answer, "review me");
        let request = server.join().unwrap();
        let (header, body) = request.split_once("\r\n\r\n").unwrap();
        assert!(header.to_ascii_lowercase().contains("authorization: bearer fake-local-test-key"));
        assert_eq!(body, expected);
    }
}
