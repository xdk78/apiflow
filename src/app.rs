use crate::http_client::{HTTPClientBuilder, HTTPMethod};
use serde::{Deserialize, Serialize};
use ureq::*;

const MAX_WIDTH_FACTOR: f32 = 3.0;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(Serialize, Deserialize, PartialEq, Clone)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct ApiFlowApp {
    url: String,
    request_body: String,
    response_body: Option<String>,
    selected_http_method: HTTPMethod,
    request_headers: Vec<(String, String)>,
}

impl Default for ApiFlowApp {
    fn default() -> Self {
        Self {
            url: "http://127.0.0.1".to_owned(),
            request_body: "".to_owned(),
            response_body: None,
            selected_http_method: HTTPMethod::Get,
            request_headers: vec![],
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
                    .selected_text(self.selected_http_method.to_string())
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
                    let mut client = HTTPClientBuilder::new()
                        .with_http_method(self.selected_http_method)
                        .with_url(self.url.clone())
                        .with_headers(
                            self.request_headers
                                .iter()
                                .map(|(key, value)| (key.clone(), value.clone()))
                                .collect(),
                        )
                        .build();

                    client.send_request(Some(self.request_body.clone()));

                    self.response_body = client
                        .response
                        .map(|response| response.into_string().unwrap_or_default())
                        .map_err(|error| error.to_string())
                        .ok();
                }
            });

            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.heading("Request Headers");
                    ui.horizontal(|ui| {
                        ui.label("Key");
                        ui.add_space(ui.available_width() / MAX_WIDTH_FACTOR);
                        ui.label("Value");
                    });
                    // TODO: add a button to remove a header
                    self.request_headers.iter_mut().for_each(|(key, value)| {
                        ui.horizontal(|ui| {
                            ui.text_edit_singleline(key);
                            ui.text_edit_singleline(value);
                        });
                    });
                    if ui.button("Add").clicked() {
                        self.request_headers.push((String::new(), String::new()));
                    }
                });
            });

            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.heading("Request body");
                    ui.add(
                        egui::TextEdit::multiline(&mut self.request_body)
                            .desired_rows(32)
                            .desired_width(ui.available_width() / MAX_WIDTH_FACTOR)
                            .code_editor(),
                    );
                });

                ui.vertical(|ui| {
                    ui.heading("Response");
                    if let Some(response_buffer) = self.response_body.as_mut() {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(response_buffer)
                                    .desired_rows(32)
                                    .desired_width(ui.available_width())
                                    .code_editor(),
                            );
                        });
                    } else {
                        ui.label(self.response_body.as_ref().unwrap_or(&String::from("")));
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
