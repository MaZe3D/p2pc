use std::string;

use egui::{Align, Layout};
use uuid::Uuid;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct App {
    // Example stuff:
    label: String,

    own_target_id: string::String,

    #[serde(skip)] // This how you opt-out of serialization of a field
    value: f32,

    messages: Vec<Message>,

    auto_scroll: bool,

    show_chats: bool,
}
#[derive(serde::Deserialize, serde::Serialize)]
struct Message {
    sender: String,
    message_id: Uuid,
    content: String,
    answer_to: AnswerTo,
}

#[derive(serde::Deserialize, serde::Serialize)]
enum AnswerTo {
    None,
    MessageId(Uuid),
}

impl Default for App {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7,
            own_target_id: "2".to_owned(),
            auto_scroll: true,
            show_chats: false,
            messages: vec![
                Message {
                    sender: "1".to_owned(),
                    message_id: uuid::Uuid::new_v4(),
                    content: "Hello".to_owned(),
                    answer_to: AnswerTo::None,
                },
                Message {
                    sender: "2".to_owned(),
                    message_id: uuid::Uuid::new_v4(),
                    content: "Hello".to_owned(),
                    answer_to: AnswerTo::None,
                },
                Message {
                    sender: "1".to_owned(),
                    message_id: uuid::Uuid::new_v4(),
                    content: "Hello".to_owned(),
                    answer_to: AnswerTo::None,
                },
                Message {
                    sender: "3".to_owned(),
                    message_id: uuid::Uuid::new_v4(),
                    content: "Hello".to_owned(),
                    answer_to: AnswerTo::None,
                },
                Message {
                    sender: "2".to_owned(),
                    message_id: uuid::Uuid::new_v4(),
                    content: "Hello".to_owned(),
                    answer_to: AnswerTo::None,
                },
                Message {
                    sender: "2".to_owned(),
                    message_id: uuid::Uuid::new_v4(),
                    content: "Hello".to_owned(),
                    answer_to: AnswerTo::None,
                },
                Message {
                    sender: "3".to_owned(),
                    message_id: uuid::Uuid::new_v4(),
                    content: "Hello".to_owned(),
                    answer_to: AnswerTo::None,
                },
                Message {
                    sender: "2".to_owned(),
                    message_id: uuid::Uuid::new_v4(),
                    content: "Hello".to_owned(),
                    answer_to: AnswerTo::None,
                },
                Message {
                    sender: "1".to_owned(),
                    message_id: uuid::Uuid::new_v4(),
                    content: "Hello".to_owned(),
                    answer_to: AnswerTo::None,
                },
                Message {
                    sender: "3".to_owned(),
                    message_id: uuid::Uuid::new_v4(),
                    content: "Hello".to_owned(),
                    answer_to: AnswerTo::None,
                },
            ],
        }
    }
}


impl App {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        // if let Some(storage) = cc.storage {
        //     return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        // }

        Default::default()
    }
}

impl eframe::App for App {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui
        let is_web = cfg!(target_arch = "wasm32");

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                if ui.selectable_label(self.show_chats, "Show Chats").clicked() {
                    self.show_chats = !self.show_chats;
                }

                egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.horizontal(|ui| {
                ui.heading("Chat");
                ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                    ui.checkbox(&mut self.auto_scroll, "Auto-scroll");
                });
            });
            egui::ScrollArea::vertical()
                .stick_to_bottom(self.auto_scroll)
                .show(ui, |ui| {
                    egui::Grid::new("chat_grid")
                        .num_columns(1)
                        .min_col_width(ui.available_width())
                        .striped(true)
                        .show(ui, |ui| {
                            for message in &self.messages {
                                ui.horizontal(|ui| {
                                    if (message.sender == self.own_target_id) {
                                        ui.with_layout(
                                            Layout::right_to_left(egui::Align::Max),
                                            |ui| {
                                                ui.label(message.content.clone());
                                            },
                                        );
                                    } else {
                                        ui.with_layout(
                                            Layout::left_to_right(egui::Align::Max),
                                            |ui| {
                                                ui.label(
                                                    egui::RichText::new(format!(
                                                        "{}:",
                                                        message.sender
                                                    ))
                                                    .italics(),
                                                );
                                                ui.label(message.content.clone());
                                            },
                                        );
                                    }
                                });
                                ui.end_row();
                            }
                        });
                });
            egui::TopBottomPanel::bottom("bottom_panel")
                .exact_height(30.)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui.button("Send").clicked() {
                                self.messages.push(Message {
                                    sender: self.own_target_id.clone(),
                                    message_id: uuid::Uuid::new_v4(),
                                    content: self.label.clone(),
                                    answer_to: AnswerTo::None,
                                });
                            }
                            egui::TextEdit::singleline(&mut self.label)
                                .min_size(ui.available_size())
                                .show(ui);
                        });
                    });
                });
            egui::SidePanel::left("chats").show_animated(ctx, self.show_chats, |ui| {
                ui.horizontal(
                    (|ui| {
                        ui.heading("Chats");
                        ui.with_layout(Layout::right_to_left(Align::Max), |ui| {
                            ui.button(" + ").clicked()
                        });
                    }),
                );
            })
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
            "https://github.com/emilk/egui/tree/main/crates/eframe",
        );
        ui.label(".");
    });
}
