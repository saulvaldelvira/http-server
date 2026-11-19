use std::collections::HashMap;

use egui::Color32;
use httpcli::{HttpRequest, http, http::HttpMethod};
use json::Json;

use crate::gui::json::json_tree;

#[derive(Default, serde::Deserialize, serde::Serialize)]
#[serde(default)]
struct Response {
    pub headers: HashMap<Box<str>, Box<str>>,
    pub status: u16,
    pub body: Box<[u8]>,
    pub version: f32,
    pub raw: String,
}

const METHODS: &[(&str, HttpMethod)] = &[
    ("GET", HttpMethod::GET),
    ("POST", HttpMethod::POST),
    ("PUT", HttpMethod::PUT),
    ("DELETE", HttpMethod::DELETE),
    ("HEAD", HttpMethod::HEAD),
    ("PATCH", HttpMethod::PATCH),
    ("CONNECT", HttpMethod::CONNECT),
    ("OPTIONS", HttpMethod::OPTIONS),
    ("TRACE", HttpMethod::TRACE),
];

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct HttpClientApp {
    url: String,

    response: Option<Response>,

    output_raw: bool,

    output_is_json: bool,

    output_json_button: bool,
    input_is_json: bool,

    input_body: String,

    #[serde(skip)]
    input_json: Json,

    #[serde(skip)]
    json_parsed: Option<Json>,

    selected_method: usize,

    error_msg: Option<String>,

    parameters: Vec<(String, String)>,
    headers: Vec<(String, String)>,

    show_help: bool,
}

impl Default for HttpClientApp {
    fn default() -> Self {
        Self {
            url: "".to_string(),
            output_raw: false,
            response: None,
            output_json_button: false,
            output_is_json: false,
            parameters: Vec::new(),
            input_is_json: false,
            input_body: String::new(),
            selected_method: 0,
            input_json: Json::Null,
            error_msg: None,
            json_parsed: None,
            show_help: false,
            headers: vec![
                ("Accept".to_string(), "*/*".to_string()),
                ("User-Agent".to_string(), "http-client".to_string()),
                ("Connection".to_string(), "close".to_string()),
                ("Accept-Encoding".to_string(), "identity".to_string()),
            ],
        }
    }
}

impl HttpClientApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        }
    }

    pub fn send_action(&mut self) -> http::Result<()> {
        let Ok((ty, port, host, url)) = httpcli::parse_url(&self.url) else {
            panic!()
        };

        let m = METHODS[self.selected_method].1;
        let mut req = HttpRequest::builder()
            .method(m)
            .url(Box::<str>::from(url))
            .version(1.1)
            .header("Host", Box::<str>::from(host))
            .body(self.input_body.clone().into_bytes())
            .header("Content-Length", format!("{}", self.input_body.len()))
            .build();

        for (k, v) in &self.parameters {
            req.params_mut()
                .insert(k.clone().into_boxed_str(), v.clone().into_boxed_str());
        }
        for (k, v) in &self.headers {
            req.set_header(k.clone().into_boxed_str(), v.clone().into_boxed_str());
        }

        let mut resp = httpcli::send_request(req, host, port as _, ty)?;

        let headers = resp.headers().clone();
        let status = resp.status();
        let version = resp.version();
        let body = resp
            .body()
            .ok()
            .flatten()
            .unwrap_or(&[])
            .to_owned()
            .into_boxed_slice();
        match str::from_utf8(&body).map(Json::deserialize) {
            Ok(Ok(json)) => {
                self.json_parsed = Some(json);
                self.output_is_json = true;
            }
            _ => {
                self.output_is_json = false;
                self.json_parsed = None;
            }
        }
        let mut raw = Vec::new();
        resp.write_raw(&mut raw)?;
        let raw = String::from_utf8_lossy_owned(raw);
        self.response = Some(Response {
            headers,
            status,
            body,
            version,
            raw,
        });
        Ok(())
    }

    fn left_panel(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            egui::ComboBox::new("method_combobox", "")
                .selected_text(METHODS[self.selected_method].0)
                .show_ui(ui, |ui| {
                    for (i, opt) in METHODS.iter().enumerate() {
                        ui.selectable_value(&mut self.selected_method, i, opt.0);
                    }
                });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("SEND").clicked() {
                    if let Err(e) = self.send_action() {
                        self.error_msg = Some(e.to_string());
                    } else {
                        self.error_msg = None;
                    }
                }
                ui.add_sized(
                    ui.available_size(),
                    egui::TextEdit::singleline(&mut self.url).hint_text("url"),
                );
            });
        });

        if let Some(err) = &self.error_msg {
            ui.colored_label(Color32::RED, err);
        }

        keyval_editor("Parameters", &mut self.parameters, ui);
        keyval_editor("Headers", &mut self.headers, ui);

        egui::CollapsingHeader::new("Body")
            .default_open(true)
            .show_unindented(ui, |ui| {
                egui::ScrollArea::new([true, true]).show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.input_body)
                            .desired_width(f32::INFINITY)
                            .desired_rows(10)
                            .interactive(true),
                    );
                });
            });
    }

    fn clear(&mut self) {
        *self = Default::default();
    }

    fn help(&mut self, ui: &mut egui::Ui) {
        egui::Window::new("Info")
            .open(&mut self.show_help)
            .collapsible(false)
            .resizable(false)
            .show(ui, |ui| {
                ui.heading("Http Client");
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Source code: ");
                    ui.hyperlink("https://github.com/saulvaldelvira/http-server");
                });
                ui.separator();
                ui.label("Copyright (C) Saúl Valdelvira (2026)");
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label(
                        "This program is free software: you can redistribute it and/or modify \
                        it under the terms of the ",
                    );
                    ui.hyperlink_to(
                        "GNU General Public License",
                        "https://www.gnu.org/licenses/gpl-3.0.en.html#license-text",
                    );
                    ui.label(
                        " as published by the \
                        Free Software Foundation, version 3.",
                    )
                });
            });
    }
}

fn keyval_editor(title: &str, entries: &mut Vec<(String, String)>, ui: &mut egui::Ui) {
    egui::CollapsingHeader::new(title)
        .default_open(false)
        .show_unindented(ui, |ui| {
            let half = (ui.available_width() - ui.spacing().item_spacing.x) * 0.5;
            let h = ui.spacing().interact_size.y;

            ui.horizontal(|ui| {
                ui.allocate_ui_with_layout(
                    [half, h].into(),
                    egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                    |ui| {
                        ui.strong("Key");
                    },
                );
                ui.allocate_ui_with_layout(
                    [half, h].into(),
                    egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                    |ui| {
                        ui.strong("Value");
                    },
                );
            });

            let mut to_remove: Option<usize> = None;

            for (i, arg) in entries.iter_mut().enumerate() {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    if ui.small_button("X").clicked() {
                        to_remove = Some(i);
                    }

                    let w = ui.available_width() / 2.0;

                    ui.add(
                        egui::TextEdit::singleline(&mut arg.1)
                            .desired_width(w)
                            .hint_text("value"),
                    );

                    ui.add(
                        egui::TextEdit::singleline(&mut arg.0)
                            .desired_width(w)
                            .hint_text("key"),
                    );
                });
            }

            if let Some(i) = to_remove {
                entries.remove(i);
            }

            if ui.button("+ Add").clicked() {
                entries.push((String::new(), String::new()));
            }
        });
}

impl eframe::App for HttpClientApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("top_panel").show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                let is_web = cfg!(target_arch = "wasm32");
                ui.menu_button("File", |ui| {
                    if !is_web && ui.button("Quit").clicked() {
                        ui.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    if ui.button("Clear").clicked() {
                        self.clear();
                    }
                    #[cfg(debug_assertions)]
                    if ui.button("Debug mode").clicked() {
                        ui.set_debug_on_hover(!ui.debug_on_hover());
                    }
                });
                ui.add_space(16.0);
                ui.menu_button("Preferences", |ui| {
                    ui.menu_button("Theme", egui::widgets::global_theme_preference_buttons);
                });
                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        self.show_help = true;
                        ui.close();
                    }
                });
            });
        });
        self.help(ui);

        ui.heading("Http Client");
        egui::Panel::left("left_panel")
            .resizable(true)
            .show(ui, |ui| self.left_panel(ui));

        egui::CentralPanel::default().show(ui, |ui| {
            let Some(response) = &self.response else {
                return;
            };

            ui.toggle_value(&mut self.output_raw, "Raw");
            if self.output_raw {
                egui::ScrollArea::new([true, true]).show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut response.raw.as_str())
                            .desired_width(f32::INFINITY)
                            .desired_rows(10)
                            .interactive(false),
                    );
                });
                return;
            }

            ui.horizontal(|ui| {
                ui.label("Status: ");
                ui.label(format!("{}", response.status));
            });

            egui::CollapsingHeader::new("Headers")
                .default_open(false)
                .show(ui, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        egui::Grid::new("headers_grid")
                            .num_columns(2)
                            .striped(true)
                            .show(ui, |ui| {
                                ui.strong("Name");
                                ui.strong("Value");
                                ui.end_row();
                                for (k, v) in response.headers.iter() {
                                    ui.label(k);
                                    ui.label(v);
                                    ui.end_row();
                                }
                            });
                    });
                });

            egui::CollapsingHeader::new("Body")
                .default_open(true)
                .show(ui, |ui| {
                    if self.output_is_json {
                        ui.toggle_value(&mut self.output_json_button, "Json");
                    }
                    if self.output_is_json && self.output_json_button {
                        if self.json_parsed.is_none() {
                            self.json_parsed = Some(
                                str::from_utf8(&self.response.as_ref().unwrap().body)
                                    .map(Json::deserialize)
                                    .unwrap()
                                    .unwrap(),
                            );
                        }
                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                json_tree(ui, self.json_parsed.as_ref().unwrap(), "", None, false);
                            });
                    } else {
                        let mut text = String::from_utf8_lossy(&response.body);
                        egui::ScrollArea::new([true, true]).show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(&mut text)
                                    .desired_width(f32::INFINITY)
                                    .desired_rows(10)
                                    .interactive(false),
                            );
                        });
                    }
                });
        });
    }
}
