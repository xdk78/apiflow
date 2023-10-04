use ureq::*;

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
enum HTTPMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
}

impl std::fmt::Display for HTTPMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HTTPMethod::Get => write!(f, "GET"),
            HTTPMethod::Post => write!(f, "POST"),
            HTTPMethod::Put => write!(f, "PUT"),
            HTTPMethod::Delete => write!(f, "DELETE"),
            HTTPMethod::Patch => write!(f, "PATCH"),
            HTTPMethod::Head => write!(f, "HEAD"),
            HTTPMethod::Options => write!(f, "OPTIONS"),
        }
    }
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct ApiFlowApp {
    url: String,
    request_body: String,
    response_body: Result<String, String>,
    #[serde(skip)]
    selected_http_method: HTTPMethod,
}

impl Default for ApiFlowApp {
    fn default() -> Self {
        Self {
            url: "http://127.0.0.1".to_owned(),
            request_body: "".to_owned(),
            response_body: Ok("".to_owned()),
            selected_http_method: HTTPMethod::Get,
        }
    }
}

impl ApiFlowApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.

        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for ApiFlowApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
                {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            _frame.close();
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("http client");

            ui.horizontal(|ui| {
                egui::ComboBox::from_id_source("http method")
                    .selected_text(format!("{:?}", self.selected_http_method))
                    .show_ui(ui, |ui| {
                        ui.style_mut().wrap = Some(false);
                        ui.set_min_width(60.0);
                        ui.selectable_value(
                            &mut self.selected_http_method,
                            HTTPMethod::Get,
                            HTTPMethod::Get.to_string(),
                        );
                        ui.selectable_value(
                            &mut self.selected_http_method,
                            HTTPMethod::Post,
                            HTTPMethod::Post.to_string(),
                        );
                        ui.selectable_value(&mut self.selected_http_method, HTTPMethod::Put, "PUT");
                        ui.selectable_value(
                            &mut self.selected_http_method,
                            HTTPMethod::Delete,
                            HTTPMethod::Delete.to_string(),
                        );
                        ui.selectable_value(
                            &mut self.selected_http_method,
                            HTTPMethod::Patch,
                            HTTPMethod::Patch.to_string(),
                        );
                        ui.selectable_value(
                            &mut self.selected_http_method,
                            HTTPMethod::Head,
                            HTTPMethod::Head.to_string(),
                        );
                        ui.selectable_value(
                            &mut self.selected_http_method,
                            HTTPMethod::Options,
                            HTTPMethod::Options.to_string(),
                        );
                    });
                ui.label("Url: ");
                ui.text_edit_singleline(&mut self.url);
                if ui.button("Send").clicked() {
                    let response = ureq::request(&self.selected_http_method.to_string(), &self.url)
                        .set("Example-Header", "header value")
                        .call();

                    match response {
                        Ok(response) => {
                            if response.status() == 200 {
                                let body = response.into_string();
                                match body {
                                    Ok(body) => {
                                        self.response_body = Ok(body);
                                    }
                                    Err(err) => {
                                        self.response_body = Err(err.to_string());
                                    }
                                }
                            } else {
                                self.response_body =
                                    Err(format!("HTTP Error: {}", response.status()));
                            }
                        }
                        Err(err) => {
                            self.response_body = Err(err.to_string());
                        }
                    }
                }
            });
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.heading("Request body");
                    ui.add(
                        egui::TextEdit::multiline(&mut self.request_body)
                            .desired_rows(32)
                            .desired_width(ui.available_width() / 3.0)
                            .code_editor(),
                    );
                });

                ui.vertical(|ui| {
                    ui.heading("Response");
                    if let Ok(response_buffer) = self.response_body.as_mut() {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(response_buffer)
                                    .desired_rows(32)
                                    .desired_width(ui.available_width())
                                    .code_editor(),
                            );
                        });
                    } else {
                        ui.label(self.response_body.as_ref().unwrap_err());
                    }
                });
            });

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}