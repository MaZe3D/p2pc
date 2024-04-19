use egui::{vec2, Align, Grid, Label, Layout, RichText, Sense};
use rand::seq::index;
use std::string;

mod chat;
use chat::Chat;

use self::chat::{ChatEditWindowContent, Contact};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct App {
    current_message: String,
    auto_scroll: bool,
    current_chat_index: Option<usize>,

    own_public_key: string::String,
    chats: Vec<Chat>,
    contacts: Vec<chat::Contact>,

    #[serde(skip)]
    show_chats: bool,

    #[serde(skip)]
    show_edit_chat: bool,

    #[serde(skip)]
    edit_contact_mode: EditMode,

    #[serde(skip)]
    contact_edit_window_content: chat::ContactEditWindowContent,
    #[serde(skip)]
    chat_edit_window_content: ChatEditWindowContent,
    #[serde(skip)]
    edit_chat_mode: EditMode,
    #[serde(skip)]
    show_contacts: bool,
}

#[derive(PartialEq)]
enum EditMode {
    None,
    New,
    Edit(usize),
    Delete(usize),
}

impl Default for App {
    fn default() -> Self {
        Self {
            // Example stuff:
            current_chat_index: Option::None,
            own_public_key: "2".to_owned(),
            auto_scroll: true,
            show_chats: false,
            show_edit_chat: false,
            show_contacts: false,
            chats: Vec::new(),
            contacts: Vec::new(),
            current_message: String::new(),
            contact_edit_window_content: Default::default(),
            chat_edit_window_content: Default::default(),
            edit_chat_mode: EditMode::None,
            edit_contact_mode: EditMode::None,
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
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

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

                if ui.selectable_label(self.show_chats, "Chats").clicked() {
                    self.show_chats = !self.show_chats;
                    if !self.show_chats {
                        if self.edit_chat_mode == EditMode::New {
                            self.show_contacts = false;
                        }
                        self.edit_chat_mode = EditMode::None;
                    }
                }
                if ui
                    .selectable_label(self.show_contacts, "Contacts")
                    .clicked()
                {
                    self.show_contacts = !self.show_contacts;
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    egui::widgets::global_dark_light_mode_switch(ui);
                });
            });
        });

        egui::TopBottomPanel::bottom("bottom_panel")
            .exact_height(30.)
            .show(ctx, |ui| {});

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.horizontal(|ui| {
                ui.heading("Chat");
                ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                    ui.checkbox(&mut self.auto_scroll, "Autoscroll");
                });
            });
            egui::ScrollArea::vertical()
                .stick_to_bottom(self.auto_scroll)
                .show(ui, |ui| {
                    if let Some(index) = self.current_chat_index {
                        egui::Grid::new("chat_grid")
                            .num_columns(1)
                            .min_col_width(ui.available_width())
                            .striped(true)
                            .show(ui, |ui| {
                                for message in self.chats[index].get_chat_messages() {
                                    ui.horizontal(|ui| {
                                        if *message.get_sender() == self.own_public_key {
                                            ui.with_layout(
                                                Layout::right_to_left(egui::Align::Max),
                                                |ui| {
                                                    let response = ui.add(
                                                        egui::Label::new(
                                                            message.get_content().clone(),
                                                        )
                                                        .sense(Sense::click()),
                                                    );
                                                    response.context_menu(|ui| {
                                                        if ui.button("↩ Answer").clicked() {
                                                            ui.close_menu();
                                                        }
                                                    })
                                                },
                                            );
                                        } else {
                                            ui.with_layout(
                                                Layout::left_to_right(egui::Align::Max),
                                                |ui| {
                                                    ui.add(egui::Label::new(
                                                        egui::RichText::new(format!(
                                                            "{}:",
                                                            message.get_sender()
                                                        ))
                                                        .italics(),
                                                    ));

                                                    let response = ui.add(
                                                        egui::Label::new(
                                                            message.get_content().clone(),
                                                        )
                                                        .sense(Sense::click()),
                                                    );
                                                    response.context_menu(|ui| {
                                                        if ui.button("↩ Answer").clicked() {
                                                            ui.close_menu();
                                                        }
                                                    })
                                                },
                                            );
                                        }
                                    });
                                    ui.end_row();
                                }
                            });
                    }
                });

            ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
                ui.horizontal(|ui| {
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.button("Send").clicked() {}
                        egui::TextEdit::singleline(&mut self.current_message)
                            .min_size(ui.available_size())
                            .show(ui);
                    });
                });
            });
        });
        egui::SidePanel::left("chats").show_animated(ctx, self.show_chats, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Chats");
                ui.with_layout(Layout::right_to_left(Align::Max), |ui| {
                    if ui.button("➕").clicked() {
                        self.chat_edit_window_content = ChatEditWindowContent::default();
                        self.edit_chat_mode = EditMode::New;
                    }
                });
            });
            egui::Grid::new("chats_grid")
                .num_columns(3)
                .striped(true)
                .min_col_width(0.)
                .show(ui, |ui| {
                    self.chats.iter().enumerate().for_each(|(index, chat)| {
                        ui.horizontal(|ui| {
                            ui.add(Label::new(chat.name.clone()));
                        });
                        if (ui.button("✏").clicked()) {
                            self.edit_chat_mode = EditMode::Edit(index);
                            self.chat_edit_window_content = ChatEditWindowContent::from_chat(chat);
                        }
                        if (ui.button("🗑").clicked()) {
                            self.edit_chat_mode = EditMode::Delete(index);
                        }
                        ui.end_row();
                    });
                    match self.edit_chat_mode {
                        EditMode::Delete(idx) => {
                            self.chats.remove(idx);
                            self.edit_chat_mode = EditMode::None;
                        }
                        _ => {}
                    }
                });
        });

        egui::SidePanel::left("edit_chats").show_animated(
            ctx,
            match self.edit_chat_mode {
                EditMode::New => true,
                EditMode::Edit(_) => true,
                _ => false,
            },
            |ui| {
                let mut title = match self.edit_chat_mode {
                    EditMode::New => "New Chat".to_string(),
                    EditMode::Edit(_) => {
                        format!("Edit Chat: {}", self.chat_edit_window_content.name).to_string()
                    }
                    _ => String::new(),
                };

                ui.horizontal(|ui| {
                    ui.add(Label::new(RichText::heading(RichText::new(title))).truncate(true));
                    ui.with_layout(Layout::right_to_left(Align::Max), |ui| {
                        if ui.button("❌").clicked() {
                            self.edit_chat_mode = EditMode::None;
                            self.show_contacts = false;
                        }
                    });
                });

                egui::Grid::new("chat_edit_grid")
                    .num_columns(2)
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("Name:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.chat_edit_window_content.name)
                                .min_size(vec2(200., 0.)),
                        );
                        ui.end_row();
                    });
                ui.separator();
                ui.heading("Participants");
                match self.edit_chat_mode {
                    EditMode::New => {
                        self.show_contacts = true;
                    }
                    _ => {}
                }
                egui::Grid::new("chat_edit_participants_grid")
                    .num_columns(match self.edit_chat_mode {
                        EditMode::New => 2,
                        _ => 1,
                    })
                    .max_col_width(0.)
                    .striped(true)
                    .show(ui, |ui| {
                        let mut chat_edit_mode_participant_edit_mode: EditMode = EditMode::None;

                        self
                                .chat_edit_window_content
                                .participants
                                .iter()
                                .enumerate()
                                .for_each(|(index, participant)| {
                                    ui.horizontal(|ui| {
                                        let contact = &self
                                            .contacts
                                            .iter()
                                            .find(|contact| contact.public_key == *participant);

                                        if (contact.is_some()) {
                                            ui.add(Label::new(
                                                RichText::new(contact.unwrap().name.clone())
                                                    .color(contact.unwrap().color.clone()),
                                            ));
                                        } else {
                                            ui.add(Label::new(
                                                RichText::new(participant.clone()).italics(),
                                            ));
                                        }

                                        match self.edit_chat_mode {
                                            EditMode::New => {
                                                if ui.button("🗑").clicked() {
                                                    chat_edit_mode_participant_edit_mode =
                                                        EditMode::Delete(index);
                                                }
                                            }
                                            _ => {}
                                        }
                                    });
                                    ui.end_row();
                                });
                    });
                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        match self.edit_chat_mode {
                            EditMode::New => {
                                let mut chat = Chat::new_chat(
                                    self.chat_edit_window_content.participants.clone(),
                                );
                                chat.name = self.chat_edit_window_content.name.clone();
                                self.chats.push(chat);
                                self.edit_chat_mode = EditMode::None;
                            }
                            EditMode::Edit(idx) => {
                                self.chats[idx].name = self.chat_edit_window_content.name.clone();
                                self.edit_chat_mode = EditMode::None;
                            }
                            _ => {}
                        }
                    }
                });
            },
        );

        egui::SidePanel::left("contacts").show_animated(ctx, self.show_contacts, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Contacts");
                ui.with_layout(Layout::right_to_left(Align::Max), |ui| {
                    if ui.button("➕").clicked() {
                        self.contact_edit_window_content =
                            chat::ContactEditWindowContent::default();
                        self.edit_contact_mode = EditMode::New;
                    }
                });
            });
            egui::Grid::new("conacts_grid")
                .num_columns(match self.edit_chat_mode {
                    EditMode::New => 3,
                    _ => 2,
                })
                .striped(true)
                .min_col_width(0.)
                .show(ui, |ui| {
                    self.contacts
                        .iter()
                        .enumerate()
                        .for_each(|(index, contact)| {
                            ui.horizontal(|ui| {
                                match self.edit_chat_mode {
                                    EditMode::New => {
                                        if (ui.button("➕").clicked()) {
                                            self.chat_edit_window_content
                                                .participants
                                                .push(contact.public_key.clone());
                                        }
                                    }
                                    _ => {}
                                }
                                ui.add(Label::new(
                                    RichText::new(contact.name.clone()).color(contact.color),
                                ));
                            });
                            if (ui.button("✏").clicked()) {
                                self.edit_contact_mode = EditMode::Edit(index);
                                self.contact_edit_window_content =
                                    chat::ContactEditWindowContent::from_contact(contact);
                            }
                            if (ui.button("🗑").clicked()) {
                                self.edit_contact_mode = EditMode::Delete(index);
                            }
                            ui.end_row();
                        });
                    match self.edit_contact_mode {
                        EditMode::Delete(idx) => {
                            self.contacts.remove(idx);
                            self.edit_contact_mode = EditMode::None;
                        }
                        _ => {}
                    }
                });
        });

        egui::SidePanel::left("edit_contacts").show_animated(
            ctx,
            match self.edit_contact_mode {
                EditMode::New => true,
                EditMode::Edit(_) => true,
                _ => false,
            },
            |ui| {
                let mut title = match self.edit_contact_mode {
                    EditMode::New => "New Contact".to_string(),
                    EditMode::Edit(idx) => {
                        format!("Edit Contact: {}", self.contact_edit_window_content.name)
                            .to_string()
                    }
                    _ => String::new(),
                };

                ui.horizontal(|ui| {
                    ui.add(Label::new(RichText::heading(RichText::new(title))).truncate(true));
                    ui.with_layout(Layout::right_to_left(Align::Max), |ui| {
                        if ui.button("❌").clicked() {
                            self.edit_contact_mode = EditMode::None;
                        }
                    });
                });

                egui::Grid::new("chat_grid")
                    .num_columns(2)
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("Name:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.contact_edit_window_content.name)
                                .min_size(vec2(200., 0.)),
                        );
                        ui.end_row();

                        ui.label("Public Key:");
                        ui.text_edit_singleline(&mut self.contact_edit_window_content.public_key);
                        ui.end_row();

                        ui.label("Color:");
                        ui.color_edit_button_rgb(&mut self.contact_edit_window_content.color);
                    });
                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        match self.edit_contact_mode {
                            EditMode::New => {
                                self.contacts.push(Contact::from_contact_window(
                                    &self.contact_edit_window_content,
                                ));
                                self.edit_contact_mode = EditMode::None;
                            }
                            EditMode::Edit(idx) => {
                                self.contacts[idx] =
                                    Contact::from_contact_window(&self.contact_edit_window_content);
                                self.edit_contact_mode = EditMode::None;
                            }
                            _ => {}
                        }
                    }
                });
            },
        );
    }
}
