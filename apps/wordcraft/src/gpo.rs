//! Windows-first GPO workbench. Document operations use the normal engine command path.
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
    message: String,
    findings: Vec<gpo::Finding>,
    changes: Vec<gpo::Change>,
    fields: BTreeMap<String, String>,
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
    consent: bool,
    pending: Option<mpsc::Receiver<Result<String, String>>>,
    revision: String,
    revision_date: String,
    revision_author: String,
    heading_lines: String,
    active_profile_editor: Option<usize>,
    merge_source: String,
    merge_output: String,
}
impl Default for Panel {
    fn default() -> Self {
        let mut panel = Self {
            open: true,
            tab: 0,
            profile: 0,
            pack: Pack::builtin().ok(),
            root: String::new(),
            catalog: Vec::new(),
            selected_template: 0,
            message: String::new(),
            findings: Vec::new(),
            changes: Vec::new(),
            fields: BTreeMap::new(),
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
            consent: false,
            pending: None,
            revision: String::new(),
            revision_date: String::new(),
            revision_author: String::new(),
            heading_lines: String::new(),
            active_profile_editor: None,
            merge_source: String::new(),
            merge_output: String::new(),
        };
        for key in ["documentNumber", "revision", "date", "title", "department", "phone", "recipient", "signatory", "position", "machineCode"] {
            panel.fields.insert(key.into(), String::new());
        }
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
impl Panel {
    fn current(&self) -> Option<Profile> {
        self.pack.as_ref()?.profiles.get(self.profile).cloned()
    }
    fn rescan(&mut self) {
        self.catalog.clear();
        self.selected_template = 0;
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
    fn result(&mut self, result: Result<Value, String>) {
        self.message = match result {
            Ok(v) => {
                if let Some(written) = v.get("written").and_then(Value::as_array) {
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
    pub fn show(&mut self, ctx: &egui::Context, app: &mut WordApp) {
        if !self.open {
            egui::Window::new("GPO launcher").title_bar(false).resizable(false).anchor(egui::Align2::RIGHT_BOTTOM, [-12.0, -30.0]).show(ctx, |ui| {
                if ui.button("GPO Word").clicked() {
                    self.open = true;
                }
            });
            return;
        }
        let mut open = true;
        egui::Window::new("GPO Word • Preview").open(&mut open).default_width(390.0).default_pos([800.0, 90.0]).show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                for (i, name) in ["เอกสาร", "ตรวจทาน", "AI", "เทมเพลต", "เครื่องมือ", "ตั้งค่า"].iter().enumerate()
                {
                    ui.selectable_value(&mut self.tab, i, *name);
                }
            });
            ui.separator();
            egui::ScrollArea::vertical().max_height(620.0).show(ui, |ui| {
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
        });
        self.open = open;
    }
    fn documents(&mut self, ui: &mut egui::Ui, app: &mut WordApp) {
        ui.label("สร้างจากสำเนาเทมเพลต ต้นฉบับจะไม่เป็นปลายทางบันทึก");
        egui::ComboBox::from_id_salt("gpo-template")
            .width(345.0)
            .selected_text(
                self.catalog
                    .get(self.selected_template)
                    .and_then(|p| p.file_name())
                    .map(|p| p.to_string_lossy().into_owned())
                    .unwrap_or("ยังไม่ได้เลือกเทมเพลต".into()),
            )
            .show_ui(ui, |ui| {
                for (i, p) in self.catalog.iter().enumerate() {
                    ui.selectable_value(&mut self.selected_template, i, p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default());
                }
            });
        if ui.button("สร้างเอกสารจากเทมเพลต").clicked() {
            if let (Some(path), Some(p)) = (self.catalog.get(self.selected_template), self.current()) {
                self.result(app.run("file.new", json!({"template":"gpo","path":path,"profile":p})));
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
        egui::CollapsingHeader::new("ข้อมูลสำหรับช่อง {{field}}").show(ui, |ui| {
            for (key, value) in &mut self.fields {
                ui.label(key);
                ui.text_edit_singleline(value);
            }
            ui.label("เว้นช่องว่างเพื่อคง {{field}} ไว้ เช่น machineCode สำหรับสร้างหลายชุด");
            if ui.button("เติมข้อมูลในช่องที่ผูกไว้").clicked() {
                let values: BTreeMap<_, _> = self.fields.iter().filter(|(_, value)| !value.trim().is_empty()).collect();
                self.result(app.run("gpo.fill", json!({"values":values})));
            }
        });
        ui.separator();
        ui.checkbox(&mut self.watermark_on, "ลายน้ำ");
        ui.text_edit_singleline(&mut self.watermark);
        ui.horizontal(|ui| {
            if ui.button("ใช้ลายน้ำกับเอกสาร").clicked() {
                self.result(app.run(
                    "design.watermark",
                    if self.watermark_on { json!({"text":self.watermark,"diagonal":true}) } else { json!({"remove":true}) },
                ));
            }
            if ui.button("ส่งออก PDF…").clicked()
                && let Some(path) = rfd::FileDialog::new().add_filter("PDF", &["pdf"]).save_file()
            {
                self.result(app.run("gpo.exportPdf", json!({"path":path,"watermark":if self.watermark_on{Some(&self.watermark)}else{None}})));
            }
        });
        ui.separator();
        egui::CollapsingHeader::new("สร้าง FM / LG หลายชุด").show(ui, |ui| {
            ui.label("ใช้ {{machineCode}} หรือช่องอื่นในเทมเพลต พร้อมไฟล์ CSV UTF-8 ที่มีชื่อคอลัมน์ตรงกัน");
            ui.text_edit_singleline(&mut self.merge_source);
            if ui.button("เลือก CSV…").clicked()
                && let Some(p) = rfd::FileDialog::new().add_filter("CSV", &["csv"]).pick_file()
            {
                self.merge_source = p.to_string_lossy().into();
            }
            ui.text_edit_singleline(&mut self.merge_output);
            if ui.button("เลือกโฟลเดอร์ผลลัพธ์…").clicked()
                && let Some(p) = rfd::FileDialog::new().pick_folder()
            {
                self.merge_output = p.to_string_lossy().into();
            }
            if ui.button("สร้าง DOCX แยกตามเครื่อง").clicked() {
                self.result(app.run("gpo.merge", json!({"csv":self.merge_source,"directory":self.merge_output,"nameField":"machineCode"})));
            }
            ui.label("Excel และ MERGEFIELD ใช้ Mailings ของตัวแก้ไขหลักได้ด้วย");
        });
    }
    fn review(&mut self, ui: &mut egui::Ui, app: &mut WordApp) {
        if ui.button("ตรวจหัวข้อ คำศัพท์ และรูปแบบรหัส").clicked()
            && let (Some(p), Some(pack)) = (self.current(), &self.pack)
        {
            match app.run("gpo.check", json!({"profile":p,"glossary":pack.glossary})) {
                Ok(v) => {
                    self.findings = serde_json::from_value(v["findings"].clone()).unwrap_or_default();
                    self.message = format!("พบ {} รายการ — ยังไม่ได้ตรวจการมีอยู่จริงในทะเบียนเอกสาร", self.findings.len());
                }
                Err(e) => self.message = e,
            }
        }
        for finding in &self.findings {
            ui.group(|ui| {
                ui.label(&finding.message);
                if !finding.found.is_empty() {
                    ui.label(format!("{} → {}", finding.found, finding.suggested));
                }
                if let Some(at) = &finding.location
                    && ui.small_button("ไปยังข้อความ").clicked()
                {
                    let _ = app.run("select.range", json!({"anchor":at,"focus":at}));
                }
            });
        }
        ui.separator();
        if ui.button("เลือกฉบับเดิมเพื่อเทียบกับฉบับที่เปิด…").clicked()
            && let Some(path) = rfd::FileDialog::new().add_filter("Word", &["doc", "docx"]).pick_file()
        {
            match app.run("gpo.compare", json!({"original":path})) {
                Ok(v) => {
                    self.changes = serde_json::from_value(v["changes"].clone()).unwrap_or_default();
                    self.message = format!("พบความต่าง {} ส่วน", self.changes.len());
                }
                Err(e) => self.message = e,
            }
        }
        for c in &self.changes {
            egui::CollapsingHeader::new(&c.section).show(ui, |ui| {
                ui.label("เดิม");
                ui.label(&c.before);
                ui.label("ใหม่");
                ui.label(&c.after);
            });
        }
        if !self.changes.is_empty() && ui.button("เตรียมให้ AI สรุป Revision History").clicked() {
            self.context = serde_json::to_string_pretty(&self.changes).unwrap_or_default();
            self.task = "สรุปการเปลี่ยนแปลงสำหรับตารางประวัติการแก้ไข ระบุเฉพาะสิ่งที่เปลี่ยนจริง แยกการแก้เนื้อหากับรูปแบบ หากบอกเหตุผลไม่ได้ห้ามเดา".into();
            self.consent = false;
            self.tab = 2;
        }
        ui.separator();
        ui.label("เพิ่มแถวประวัติการแก้ไขหลังตรวจทานคำสรุปแล้ว");
        ui.label("Revision");
        ui.text_edit_singleline(&mut self.revision);
        ui.label("วันที่บังคับใช้");
        ui.text_edit_singleline(&mut self.revision_date);
        ui.label("ผู้จัดทำ (ถ้าตารางมีคอลัมน์นี้)");
        ui.text_edit_singleline(&mut self.revision_author);
        ui.text_edit_multiline(&mut self.answer);
        if ui.button("เพิ่มแถวจากข้อความที่ตรวจแล้ว").clicked() {
            self.result(app.run(
                "gpo.revision",
                json!({"revision":self.revision,"date":self.revision_date,"summary":self.answer,"author":self.revision_author}),
            ));
        }
    }
    fn ai(&mut self, ui: &mut egui::Ui, app: &mut WordApp) {
        ui.label("งานที่ต้องการให้ช่วย");
        ui.text_edit_multiline(&mut self.task);
        ui.horizontal_wrapped(|ui| {
            if ui.button("ใช้ข้อความจากเอกสาร").clicked() {
                self.context = gpo::document_text(&app.session.doc);
                self.consent = false;
            }
            if ui.button("ใช้เฉพาะข้อความที่เลือก").clicked() {
                let (a, b) = app.session.sel.ordered();
                self.context = app.session.doc.copy_range(&a, &b).plain_text();
                self.consent = false;
            }
        });
        ui.label("เนื้อหาที่จะส่งไปยัง API — แก้ไขหรือลบข้อมูลได้ก่อนส่ง");
        if ui.add(egui::TextEdit::multiline(&mut self.context).desired_rows(7)).changed() {
            self.consent = false;
        }
        ui.checkbox(&mut self.consent, "ส่งเนื้อหาที่แสดงด้านบนไปยัง API ที่ตั้งค่าไว้");
        if ui.add_enabled(self.pending.is_none() && self.consent, egui::Button::new("ให้ AI ช่วยเขียน")).clicked() {
            self.start_ai(false);
        }
        if self.pending.is_some() {
            ui.spinner();
            ui.label("กำลังรอคำตอบ…");
        }
        ui.label("คำตอบที่แก้ไขได้");
        ui.add(egui::TextEdit::multiline(&mut self.answer).desired_rows(8));
        if ui.button("แทรกข้อความที่ตรวจแล้ว ณ ตำแหน่งเลือก").clicked() {
            self.result(app.run("text.insert", json!({"text":self.answer})));
        }
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
            ui.add(egui::Slider::new(&mut p.font_size, 10.0..=24.0).text("ขนาด"));
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
        for key in self.fields.keys().cloned().collect::<Vec<_>>() {
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
        ui.text_edit_multiline(&mut self.utility);
        if ui.button("แปลง").clicked() {
            let mode = ["thaiDigits", "arabicDigits", "baht", "date"].get(self.utility_mode).copied().unwrap_or("date");
            match app.run("gpo.thai", json!({"mode":mode,"text":self.utility,"buddhistInput":self.buddhist_input})) {
                Ok(v) => self.utility_output = v["text"].as_str().unwrap_or("").into(),
                Err(e) => self.message = e,
            }
        }
        ui.text_edit_multiline(&mut self.utility_output);
        if ui.button("แทรกผลลัพธ์").clicked() {
            self.result(app.run("text.insert", json!({"text":self.utility_output})));
        }
    }
    fn settings(&mut self, ui: &mut egui::Ui) {
        ui.label("โฟลเดอร์ docsStorage");
        ui.text_edit_singleline(&mut self.root);
        if ui.button("เลือกโฟลเดอร์…").clicked()
            && let Some(p) = rfd::FileDialog::new().pick_folder()
        {
            self.root = p.to_string_lossy().into();
            self.rescan();
        }
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
        ui.label("พจนานุกรม — รายการเริ่มต้นเป็นเพียงคำผิดทั่วไป เพิ่มศัพท์ GMP ของทีมได้ที่นี่");
        if let Some(pack) = &mut self.pack {
            let mut remove = None;
            for (i, rule) in pack.glossary.iter_mut().enumerate() {
                ui.push_id(i, |ui| {
                    ui.horizontal(|ui| {
                        ui.add(egui::TextEdit::singleline(&mut rule.find).desired_width(135.0));
                        ui.label("→");
                        ui.add(egui::TextEdit::singleline(&mut rule.preferred).desired_width(135.0));
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
fn complete(url: &str, model: &str, key: Password, (system, user): (String, String)) -> Result<String, String> {
    validate_url(url)?;
    let body=json!({"model":model,"messages":[{"role":"system","content":system},{"role":"user","content":user}],"temperature":0.2,"max_tokens":2048,"stream":false}).to_string();
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
