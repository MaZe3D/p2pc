use egui::{vec2, Align, Button, Label, Layout, RichText, Sense};
use std::{fmt::format, string, thread::LocalKey};
use uuid::Uuid;

mod chat;
use chat::Chat;

use self::chat::{ChatEditWindowContent, Contact, ContactEditWindowContent};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct App {
    current_message: String,
    current_message_answer_to: Option<Uuid>,
    auto_scroll: bool,
    current_chat_index: Option<usize>,

    own_public_key: string::String,
    chats: Vec<Chat>,
    contacts: Vec<chat::Contact>,

    drop_chat_messages_from_unkown: bool,
    theme: Theme,

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

#[derive(serde::Deserialize, serde::Serialize, PartialEq)]
enum Theme {
    LATTE,
    FRAPPE,
    MACCHIATO,
    MOCHA,
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
            own_public_key: "MFwwDQYJKoZIhvcNAQEBBQADSwAwSAJBAI8bPMUvxS90itiFYHak9ZnqzuhqCr0o
7UNVByX+EbBrR+JXpIIiUTvEfRQvxWKrLjRnD/ObaTBaV4QrC6wJpp0CAwEAAQ=="
                .to_owned(),
            auto_scroll: true,
            show_chats: false,
            show_edit_chat: false,
            show_contacts: false,
            drop_chat_messages_from_unkown: false,
            chats: Vec::new(),
            contacts: Vec::new(),
            current_message: String::new(),
            current_message_answer_to: None,
            contact_edit_window_content: Default::default(),
            chat_edit_window_content: Default::default(),
            edit_chat_mode: EditMode::None,
            edit_contact_mode: EditMode::None,
            theme: Theme::MACCHIATO,
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

        cc.egui_ctx.set_zoom_factor(1.5);

        setup_custom_fonts(&cc.egui_ctx);

        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }

    fn update_theme(&mut self, ctx: &egui::Context) {
        match self.theme {
            Theme::LATTE => catppuccin_egui::set_theme(&ctx, catppuccin_egui::LATTE),
            Theme::FRAPPE => catppuccin_egui::set_theme(&ctx, catppuccin_egui::FRAPPE),
            Theme::MACCHIATO => catppuccin_egui::set_theme(&ctx, catppuccin_egui::MACCHIATO),
            Theme::MOCHA => catppuccin_egui::set_theme(&ctx, catppuccin_egui::MOCHA),
        }
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
                    ui.menu_button("🎨", |ui| {
                        ui.heading("Theme");
                        ui.selectable_value(&mut self.theme, Theme::LATTE, "Latte");
                        ui.selectable_value(&mut self.theme, Theme::FRAPPE, "Frappe");
                        ui.selectable_value(&mut self.theme, Theme::MACCHIATO, "Macchiato");
                        ui.selectable_value(&mut self.theme, Theme::MOCHA, "Mocha");
                    });
                    self.update_theme(ctx);
                });
            });
        });

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("ptpc");
                ui.separator();
                if ui
                    .add(egui::Label::new(&self.own_public_key).truncate(true))
                    .on_hover_text(format!("Click to copy."))
                    .clicked()
                {
                    if !is_web {
                        ctx.output_mut(|o| o.copied_text = self.own_public_key.clone());
                    }
                };
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's

            match self.current_chat_index {
                Some(current_chat_index) => match self.chats.get_mut(current_chat_index) {
                    Some(current_chat) => {
                        ui.horizontal(|ui| {
                            ui.heading(RichText::new("Chat:"));
                            ui.heading(current_chat.name.clone());
                            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                                ui.checkbox(&mut self.auto_scroll, "Autoscroll");
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
                                        for message in current_chat.get_chat_messages() {
                                            let sender = self.contacts.iter().find(|contact| {
                                                contact.public_key == *message.get_sender()
                                            });
                                            let sender_is_user =
                                                message.get_sender() == &self.own_public_key;
                                            let layout = if sender_is_user {
                                                Layout::right_to_left(Align::Max)
                                            } else {
                                                Layout::left_to_right(Align::Max)
                                            };

                                            ui.vertical(|ui| {
                                                ui.with_layout(layout, |ui| {
                                                    ui.label(
                                                        RichText::new(
                                                            message
                                                                .recieved_time
                                                                .format("%Y-%m-%d %H:%M:%S")
                                                                .to_string(),
                                                        )
                                                        .color(egui::Color32::GRAY)
                                                        .size(8.),
                                                    );
                                                    if let Some(answer_to_id) = message.get_answer_to() {
                                                        if let Some(answer_to_message) = current_chat.get_message_from_id(answer_to_id) {
                                                            ui.add(Label::new(RichText::new(format!("{} ⮪", answer_to_message.get_content()).to_string()).size(10.)));
                                                        }
                                                    }
                                                });
                                                ui.with_layout(layout, |ui| {
                                                    match sender {
                                                        Some(contact) => {
                                                            ui.add(Label::new(
                                                                RichText::new(contact.name.clone())
                                                                    .color(contact.color)
                                                                    .italics()
                                                                    .size(10.),
                                                            ))
                                                            .on_hover_text(
                                                                contact.public_key.clone(),
                                                            );
                                                        }
                                                        None => {
                                                            if !sender_is_user {
                                                                let sender_label_response = ui
                                                                    .add(Label::new(
                                                                        RichText::new(format!(
                                                                            "{}...",
                                                                            &message.get_sender()
                                                                                [..8]
                                                                        ))
                                                                        .color(egui::Color32::RED),
                                                                    ))
                                                                    .on_hover_text(format!(
                                                                        "{}",
                                                                        &message.get_sender()
                                                                    ));

                                                                sender_label_response.context_menu(
                                                                    |ui| {
                                                                        if ui
                                                                            .button("➕ Add to contacts")
                                                                            .clicked()
                                                                        {
                                                                            ui.close_menu();
                                                                            self.contact_edit_window_content = ContactEditWindowContent::default();
                                                                            self.contact_edit_window_content.public_key = message.get_sender().clone();
                                                                            self.edit_contact_mode = EditMode::New;
                                                                        }
                                                                    },
                                                                );
                                                            }
                                                        }
                                                    }
                                                    let message_label_response = ui.add(
                                                        Label::new(RichText::new(
                                                            message.get_content().clone(),
                                                        ))
                                                        .wrap(true),
                                                    );

                                                    message_label_response.context_menu(|ui| {
                                                        if ui.button("⮪ Answer").clicked() {
                                                            ui.close_menu();
                                                            self.current_message_answer_to = Some(message.get_message_id().clone());
                                                        }
                                                    });
                                                });
                                            });
                                            ui.end_row();
                                        }
                                    });
                            });

                        ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
                            ui.horizontal(|ui| {
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if ui.add_enabled(!self.current_message.trim().is_empty(), Button::new("Send ➡")).clicked() {
                                        current_chat.new_message(
                                            //"0x1234".to_string(),
                                            self.own_public_key.clone(),
                                            self.current_message.trim().to_string(),
                                            self.current_message_answer_to,
                                        );
                                        self.current_message.clear();
                                        self.current_message_answer_to = None;
                                    }
                                    egui::TextEdit::singleline(&mut self.current_message)
                                    .min_size(ui.available_size())
                                    .show(ui);
                                });

                            });
                            if let Some(answer_to) = self.current_message_answer_to {
                                ui.with_layout(Layout::right_to_left(Align::Max), |ui| {
                                    if ui.button("❌").clicked() {
                                        self.current_message_answer_to = None;
                                    }
                                    if let Some(answer_to_message) = current_chat.get_message_from_id(&answer_to) {
                                        ui.label(format!("{} ⮪", answer_to_message.get_content()));
                                    }
                                });
                        }
                        ui.separator();
                        });
                    }
                    None => {
                        self.current_chat_index = None;
                    }
                },
                None => {
                    ui.heading(
                        RichText::new("No chat selected")
                            .italics()
                            .color(egui::Color32::RED),
                    );
                }
            }
        });

        egui::SidePanel::left("chats")
            .min_width(50.)
            .show_animated(ctx, self.show_chats, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Chats");
                    ui.with_layout(Layout::right_to_left(Align::Max), |ui| {
                        if ui.button("➕").clicked() {
                            self.chat_edit_window_content = ChatEditWindowContent::default();
                            self.edit_chat_mode = EditMode::New;
                        }
                    });
                });
                egui::ScrollArea::vertical().show(ui, |ui| {
                    egui::Grid::new("chats_grid")
                        .num_columns(3)
                        .striped(true)
                        .min_col_width(0.)
                        .show(ui, |ui| {
                            self.chats.iter().enumerate().for_each(|(index, chat)| {
                                ui.horizontal(|ui| {
                                    if ui
                                        .add(egui::SelectableLabel::new(
                                            self.current_chat_index == Some(index),
                                            chat.name.clone(),
                                        ))
                                        .clicked()
                                    {
                                        self.current_chat_index = Some(index);
                                        self.show_chats = false;
                                    }
                                });
                                if (ui.button("✏").clicked()) {
                                    self.edit_chat_mode = EditMode::Edit(index);
                                    self.chat_edit_window_content =
                                        ChatEditWindowContent::from_chat(chat);
                                }
                                if (ui.button("🗑").clicked()) {
                                    self.edit_chat_mode = EditMode::Delete(index);
                                }
                                ui.end_row();
                            });
                        });
                    match self.edit_chat_mode {
                        EditMode::Delete(idx) => match self.current_chat_index {
                            Some(selected_chat_index) => {
                                let selected_chat_id =
                                    self.chats[selected_chat_index].get_chat_id().clone();
                                self.chats.remove(idx);

                                self.current_chat_index = self
                                    .chats
                                    .iter()
                                    .position(|chat| *chat.get_chat_id() == selected_chat_id);
                                self.edit_chat_mode = EditMode::None;
                            }
                            None => {}
                        },
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
                    .min_col_width(0.)
                    .striped(true)
                    .show(ui, |ui| {
                        let mut chat_edit_mode_participant_edit_mode: EditMode = EditMode::None;

                        self.chat_edit_window_content
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
                                });
                                match self.edit_chat_mode {
                                    EditMode::New => {
                                        if ui.button("🗑").clicked() {
                                            chat_edit_mode_participant_edit_mode =
                                                EditMode::Delete(index);
                                        }
                                    }
                                    _ => {}
                                }
                                ui.end_row();
                            });
                        match chat_edit_mode_participant_edit_mode {
                            EditMode::Delete(idx) => {
                                self.chat_edit_window_content.participants.remove(idx);
                            }
                            _ => {}
                        }
                    });
                ui.separator();
                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(
                            self.chat_edit_window_content.participants.len() > 0
                                && !self.chat_edit_window_content.name.is_empty(),
                            egui::Button::new("Save"),
                        )
                        .on_disabled_hover_text("Please add at least one participant and a name.")
                        .clicked()
                    {
                        match self.edit_chat_mode {
                            EditMode::New => {
                                let mut chat = Chat::new_chat(
                                    self.chat_edit_window_content.participants.clone(),
                                );
                                chat.name = self.chat_edit_window_content.name.clone();
                                self.chats.push(chat);
                                self.edit_chat_mode = EditMode::None;
                                if self.current_chat_index.is_none() {
                                    self.current_chat_index = Some(self.chats.len() - 1);
                                }
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

        egui::SidePanel::left("contacts")
            .min_width(100.)
            .show_animated(ctx, self.show_contacts, |ui| {
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
                                            if (ui.button("➕").clicked()
                                                && !self
                                                    .chat_edit_window_content
                                                    .participants
                                                    .contains(&contact.public_key))
                                            {
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
                    if ui
                        .add_enabled(
                            !self.contact_edit_window_content.public_key.is_empty()
                                && !self.contact_edit_window_content.name.is_empty(),
                            egui::Button::new("Save"),
                        )
                        .on_disabled_hover_text("Please add at least one participant and a name.")
                        .clicked()
                    {
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

fn setup_custom_fonts(ctx: &egui::Context) {
    // Start with the default fonts (we will be adding to them rather than replacing them).
    let mut fonts = egui::FontDefinitions::default();

    // Install my own font (maybe supporting non-latin characters).
    // .ttf and .otf files supported.
    fonts.font_data.insert(
        "sharetech".to_owned(),
        egui::FontData::from_static(include_bytes!("../../assets/ShareTech.ttf")),
    );

    fonts.font_data.insert(
        "sharetechmono".to_owned(),
        egui::FontData::from_static(include_bytes!("../../assets/ShareTechMono.ttf")),
    );

    // Put my font first (highest priority) for proportional text:
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "sharetech".to_owned());

    // Put my font as last fallback for monospace:
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("sharetechmono".to_owned());

    // Tell egui to use these fonts:
    ctx.set_fonts(fonts);
}
