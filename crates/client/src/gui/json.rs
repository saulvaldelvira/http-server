use json::Json;

pub fn json_tree(
    ui: &mut egui::Ui,
    value: &Json,
    path: &str,
    force: Option<bool>,
    default_open: bool,
) {
    match value {
        Json::Object(map) if !map.is_empty() => {
            for (key, val) in map {
                ui.horizontal(|ui| {
                    ui.monospace(format!("{key}:"));
                    if is_container(val) {
                        let child = format!("{path}/{key}");
                        egui::CollapsingHeader::new(format!("{}", summarise(val)))
                            .id_salt(&child) // unique per path
                            .default_open(default_open)
                            .open(force) // None = user-controlled
                            .show(ui, |ui| json_tree(ui, val, &child, force, default_open));
                    } else {
                        json_leaf(ui, val);
                    }
                });
            }
        }
        Json::Array(arr) if !arr.is_empty() => {
            for (i, val) in arr.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.monospace(format!("[{i}]:"));
                    if is_container(val) {
                        let child = format!("{path}[{i}]");
                        egui::CollapsingHeader::new(format!("[{i}]  {}", summarise(val)))
                            .id_salt(&child)
                            .default_open(default_open)
                            .open(force)
                            .show(ui, |ui| json_tree(ui, val, &child, force, default_open));
                    } else {
                        json_leaf(ui, val);
                    }
                });
            }
        }
        _ => json_leaf(ui, value), // empty object/array or leaf at root
    }
}

fn is_container(v: &Json) -> bool {
    match v {
        Json::Object(m) => !m.is_empty(),
        Json::Array(a) => !a.is_empty(),
        _ => false,
    }
}

fn summarise(v: &Json) -> String {
    match v {
        Json::Object(m) => format!("{{…{} keys}}", m.len()),
        Json::Array(a) => format!("[…{} items]", a.len()),
        _ => String::new(),
    }
}

fn json_leaf(ui: &mut egui::Ui, v: &Json) {
    let (color, text) = match v {
        Json::Null => (egui::Color32::GRAY, "null".to_owned()),
        Json::True => (egui::Color32::from_rgb(86, 156, 214), "true".to_owned()),
        Json::False => (egui::Color32::from_rgb(86, 156, 214), "False".to_owned()),
        Json::Number(n) => (egui::Color32::from_rgb(181, 206, 168), n.to_string()),
        Json::String(s) => (egui::Color32::from_rgb(206, 145, 120), format!("\"{s}\"")),
        Json::Object(_) | Json::Array(_) => {
            // empty container renders as a small marker
            (
                ui.visuals().weak_text_color(),
                if matches!(v, Json::Object(_)) {
                    "{}"
                } else {
                    "[]"
                }
                .to_owned(),
            )
        }
    };
    ui.colored_label(color, text);
}
