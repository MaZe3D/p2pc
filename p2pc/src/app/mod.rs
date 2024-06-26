use std::str::FromStr;

use egui::{vec2, Align, Button, Label, Layout, RichText, TextEdit};
use egui::{Grid, ScrollArea};

use libp2p::Multiaddr;
use uuid::Uuid;

mod chat;
use chat::Chat;
use chat::Chats;
use chat::Contacts;

mod keypair_wrapper;

use self::chat::{ChatEditWindowContent, Contact, ContactEditWindowContent};
use clap::Parser as _;

/// Start or connect to an existing p2pc network
#[derive(clap::Parser, Debug)]
struct CliArguments {
    /// Initial peers to connect to
    #[arg(short, long, num_args(0..), value_name="MULTIADDRESS")]
    peer_addresses: Vec<libp2p::Multiaddr>,

    /// Interfaces to listen on
    #[arg(short, long, num_args(0..), value_name="MULTIADDRESS", default_values = vec!["/ip4/0.0.0.0/tcp/0", "/ip6/::/tcp/0"])]
    listen_addresses: Vec<libp2p::Multiaddr>,
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct App {
    current_message: String,
    current_message_answer_to: Option<Uuid>,
    auto_scroll: bool,
    current_chat_id: Option<uuid::Uuid>,

    keypair: keypair_wrapper::Keypair,
    chats: std::sync::Arc<std::sync::Mutex<Chats>>,
    contacts: Contacts,

    #[serde(skip)]
    listen_addresses: std::sync::Arc<std::sync::Mutex<Vec<Multiaddr>>>,

    drop_chat_messages_from_unkown: bool,
    theme: Theme,

    settings: Settings,

    #[serde(skip)]
    show_chats: bool,

    #[serde(skip)]
    show_edit_chat: bool,

    #[serde(skip)]
    show_settings: bool,

    #[serde(skip)]
    edit_contact_mode: EditMode<String>,

    #[serde(skip)]
    contact_edit_window_content: chat::ContactEditWindowContent,
    #[serde(skip)]
    chat_edit_window_content: ChatEditWindowContent,
    #[serde(skip)]
    edit_chat_mode: EditMode<uuid::Uuid>,
    #[serde(skip)]
    show_contacts: bool,

    #[serde(skip)]
    p2pc: Option<p2pc_lib::P2pc>,
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq)]
enum Theme {
    LATTE,
    FRAPPE,
    MACCHIATO,
    MOCHA,
}

#[derive(PartialEq, Clone, Copy)]
enum EditMode<T> {
    None,
    New,
    Edit(T),
    Delete(T),
}

impl Default for App {
    fn default() -> Self {
        Self {
            // Example stuff:
            current_chat_id: Option::None,
            auto_scroll: true,
            show_chats: false,
            show_edit_chat: false,
            show_contacts: false,
            show_settings: false,
            settings: Settings::default(),
            drop_chat_messages_from_unkown: false,
            chats: std::sync::Arc::new(std::sync::Mutex::new(Chats::default())),
            contacts: Contacts::default(),
            current_message: String::new(),
            current_message_answer_to: None,
            contact_edit_window_content: Default::default(),
            chat_edit_window_content: Default::default(),
            edit_chat_mode: EditMode::None,
            edit_contact_mode: EditMode::None,
            theme: Theme::MACCHIATO,
            p2pc: None,
            keypair: Default::default(),
            listen_addresses: std::sync::Arc::new(std::sync::Mutex::new(vec![])),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
struct Settings {
    peers: Vec<Multiaddr>,
    listen_addresses: Vec<Multiaddr>,

    #[serde(skip)]
    current_peer: String,
    #[serde(skip)]
    current_peer_is_valid: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            peers: Vec::new(),
            listen_addresses: Vec::new(),
            current_peer: String::new(),
            current_peer_is_valid: false,
        }
    }
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_zoom_factor(1.5);
        setup_custom_fonts(&cc.egui_ctx);

        let mut app: Self = match cc.storage {
            Some(storage) => eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default(),
            None => Default::default(),
        };

        let mut p2pc = {
            let egui_ctx = cc.egui_ctx.clone();
            let listen_addresses = app.listen_addresses.clone();
            let chats = app.chats.clone();
            p2pc_lib::P2pc::new(app.keypair.get_keypair(), move |event| {
                Self::handle_p2pc_event(event, &egui_ctx, listen_addresses.clone(), chats.clone())
            })
            .expect("could not initialize p2pc")
        };

        let args = CliArguments::parse();
        log::info!("{:?}", args);

        for address in args.peer_addresses {
            p2pc.execute(p2pc_lib::Action::Dial(address)).ok();
        }

        for address in args.listen_addresses {
            p2pc.execute(p2pc_lib::Action::ListenOn(address)).ok();
        }

        for address in &app.settings.peers {
            p2pc.execute(p2pc_lib::Action::Dial(address.clone())).ok();
        }

        app.p2pc = Some(p2pc);
        app
    }

    fn handle_p2pc_event(
        event: p2pc_lib::Event,
        egui_ctx: &egui::Context,
        listen_addresses: std::sync::Arc<std::sync::Mutex<Vec<Multiaddr>>>,
        chats: std::sync::Arc<std::sync::Mutex<Chats>>,
    ) {
        match event {
            p2pc_lib::Event::NewListenAddress(address) => {
                log::info!("listening on {}", address);
                let mut listen_addresses = listen_addresses.lock().unwrap();
                listen_addresses.push(address);
                egui_ctx.request_repaint();
            }
            p2pc_lib::Event::ActionResult(action_result) => match action_result {
                p2pc_lib::ActionResult::SendMessage {
                    message_id: _,
                    chat_id: _,
                    optional_errors,
                } => {
                    if optional_errors.is_empty() {
                        log::info!("sending message failed");
                    } else {
                        optional_errors.iter().for_each(|optional_error| {
                            if let Some(error) = optional_error {
                                log::error!("{}", error);
                            }
                        });
                    }
                }
            },
            p2pc_lib::Event::MessageReceived(p2pc_lib::ChatMessage {
                participants,
                content,
                id,
                chat_id,
                answer_to,
            }) => {
                if let Some(sender) = participants.last() {
                    let mut chats = chats.lock().unwrap();
                    if let Some(mut chat) = chats.remove_chat(&chat_id) {
                        chat.insert_message(sender.clone(), content, answer_to, id);
                        chats.add_chat(chat);
                    } else {
                        let sender = sender.clone();
                        let mut chat = Chat::new_incoming_chat(participants, chat_id);
                        chat.insert_message(sender.clone(), content, answer_to, id);
                        chats.add_chat(chat);
                    }
                }
                egui_ctx.request_repaint();
            }
        }
    }

    fn update_theme(&mut self, ctx: &egui::Context) {
        match self.theme {
            Theme::LATTE => catppuccin_egui::set_theme(ctx, catppuccin_egui::LATTE),
            Theme::FRAPPE => catppuccin_egui::set_theme(ctx, catppuccin_egui::FRAPPE),
            Theme::MACCHIATO => catppuccin_egui::set_theme(ctx, catppuccin_egui::MACCHIATO),
            Theme::MOCHA => catppuccin_egui::set_theme(ctx, catppuccin_egui::MOCHA),
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
        let own_public_key_base_64 = self.keypair.get_peer_id();

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
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
                    if ui.selectable_label(self.show_settings, "⚙").clicked() {
                        self.show_settings = !self.show_settings;
                    };
                    self.update_theme(ctx);
                });
            });
        });

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                ui.label("p2pc");
                ui.separator();
                if ui
                    .add(egui::Label::new(own_public_key_base_64.clone()))
                    .on_hover_text("Click to copy Public Key".to_string())
                    .clicked()
                {
                    ctx.output_mut(|o| o.copied_text = own_public_key_base_64.clone());
                };
            });
        });

        if let Some(chat_id) = self.current_chat_id {
            egui::TopBottomPanel::bottom("Send Chat Message")
                .show_separator_line(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.vertical(|ui| {
                        ui.with_layout(Layout::right_to_left(Align::Max), |ui| {
                            let button_response = ui.add_enabled(
                                !self.current_message.trim().is_empty(),
                                Button::new("Send ➡").min_size(vec2(0., ui.available_height())),
                            );

                            let textedit_response = ui.add(
                                egui::TextEdit::singleline(&mut self.current_message)
                                    .desired_rows(1)
                                    .hint_text(
                                        RichText::new("Type a message...")
                                            .color(egui::Color32::GRAY),
                                    )
                                    .min_size(ui.available_size()),
                            );

                            // send message
                            if button_response.clicked()
                                || (textedit_response.lost_focus()
                                    && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                            {
                                if let Some(p2pc) = &mut self.p2pc {
                                    self.chats.lock().unwrap().send_message(
                                        &chat_id,
                                        p2pc,
                                        own_public_key_base_64.clone(),
                                        self.current_message.trim().to_string(),
                                        self.current_message_answer_to,
                                    );
                                    self.current_message.clear();
                                    self.current_message_answer_to = None;
                                }

                                textedit_response.request_focus();
                            }
                        });
                    });
                });
            if let Some(answer_to) = self.current_message_answer_to {
                egui::TopBottomPanel::bottom("answer_to_message")
                    .show_separator_line(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.with_layout(Layout::right_to_left(Align::Max), |ui| {
                            if ui.button("❌").clicked() {
                                self.current_message_answer_to = None;
                            }
                            if let Some(current_chat) =
                                self.chats.lock().unwrap().get_chat(&chat_id)
                            {
                                if let Some(answer_to_message) =
                                    current_chat.get_message_from_id(&answer_to)
                                {
                                    ui.add(
                                        Label::new(format!(
                                            "{} ⮪",
                                            answer_to_message.get_content()
                                        ))
                                        .truncate(true),
                                    );
                                }
                            }
                        });
                    });
            }
        }

        egui::SidePanel::left("chats")
            .min_width(60.)
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
                        .striped(false)
                        .min_col_width(0.)
                        .show(ui, |ui| {
                            self.chats.lock().unwrap().get_chats().iter().for_each(
                                |(&chat_id, chat)| {
                                    ui.horizontal(|ui| {
                                        if ui
                                            .add(egui::SelectableLabel::new(
                                                self.current_chat_id == Some(chat_id),
                                                chat.name.clone(),
                                            ))
                                            .clicked()
                                        {
                                            self.current_chat_id = Some(chat_id);
                                        }
                                    });
                                    if ui.button("✏").clicked() {
                                        self.edit_chat_mode = EditMode::Edit(chat_id);
                                        self.chat_edit_window_content =
                                            ChatEditWindowContent::from_chat(chat);
                                    }
                                    if ui.button("🗑").clicked() {
                                        self.edit_chat_mode = EditMode::Delete(chat_id);
                                    }
                                    ui.end_row();
                                },
                            );
                        });
                    match self.edit_chat_mode {
                        EditMode::Delete(selected_chat_id) => {
                            if let Some(current_chat_id) = self.current_chat_id {
                                if selected_chat_id == current_chat_id {
                                    self.current_chat_id = None;
                                }
                            }
                            self.chats.lock().unwrap().remove_chat(&selected_chat_id);
                            self.edit_chat_mode = EditMode::None;
                        }
                        _ => {}
                    }
                });
            });

        egui::SidePanel::left("edit_chats")
            .min_width(60.)
            .show_animated(
                ctx,
                match self.edit_chat_mode {
                    EditMode::New => true,
                    EditMode::Edit(_) => true,
                    _ => false,
                },
                |ui| {
                    let title = match self.edit_chat_mode {
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
                        .striped(false)
                        .min_col_width(0.)
                        .show(ui, |ui| {
                            ui.label("Name:");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.chat_edit_window_content.name)
                                    .min_size(vec2(50., 0.)),
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
                        .striped(false)
                        .show(ui, |ui| {
                            let mut chat_edit_mode_participant_edit_mode: EditMode<usize> =
                                EditMode::None;

                            self.chat_edit_window_content
                                .participants
                                .iter()
                                .enumerate()
                                .for_each(|(index, participant)| {
                                    ui.horizontal(|ui| {
                                        let contact = &self.contacts.get_contact(participant);

                                        if contact.is_some() {
                                            ui.add(Label::new(
                                                RichText::new(contact.unwrap().name.clone())
                                                    .color(contact.unwrap().color),
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
                                !self.chat_edit_window_content.participants.is_empty()
                                    && !self.chat_edit_window_content.name.is_empty(),
                                egui::Button::new("Save"),
                            )
                            .on_disabled_hover_text(
                                "Please add at least one participant and a name.",
                            )
                            .clicked()
                        {
                            let mut self_chats = self.chats.lock().unwrap();
                            match self.edit_chat_mode {
                                EditMode::New => {
                                    let mut chat = Chat::new_chat(
                                        self.chat_edit_window_content.participants.clone(),
                                    );
                                    chat.name = self.chat_edit_window_content.name.clone();
                                    self.edit_chat_mode = EditMode::None;
                                    if self.current_chat_id.is_none() {
                                        self.current_chat_id = Some(*chat.get_chat_id());
                                    }
                                    self_chats.add_chat(chat);
                                }
                                EditMode::Edit(chat_id) => {
                                    if let Some(mut chat) = self_chats.remove_chat(&chat_id) {
                                        chat.name = self.chat_edit_window_content.name.clone();
                                        self_chats.add_chat(chat);
                                    }
                                    self.edit_chat_mode = EditMode::None;
                                }
                                _ => {}
                            }
                        }
                    });
                },
            );

        egui::SidePanel::left("contacts")
            .min_width(60.)
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
                egui::ScrollArea::vertical().show(ui, |ui| {
                    egui::Grid::new("conacts_grid")
                        .num_columns(match self.edit_chat_mode {
                            EditMode::New => 3,
                            _ => 2,
                        })
                        .striped(false)
                        .min_col_width(0.)
                        .show(ui, |ui| {
                            self.contacts.get_contacts().iter().for_each(
                                |(public_key, contact)| {
                                    ui.horizontal(|ui| {
                                        match self.edit_chat_mode {
                                            EditMode::New => {
                                                if ui.button("➕").clicked()
                                                    && !self
                                                        .chat_edit_window_content
                                                        .participants
                                                        .contains(&contact.public_key)
                                                {
                                                    self.chat_edit_window_content
                                                        .participants
                                                        .push(contact.public_key.clone());
                                                }
                                            }
                                            _ => {}
                                        }
                                        ui.add(Label::new(
                                            RichText::new(contact.name.clone())
                                                .color(contact.color),
                                        ));
                                    });
                                    if ui.button("✏").clicked() {
                                        self.edit_contact_mode = EditMode::Edit(public_key.clone());
                                        self.contact_edit_window_content =
                                            chat::ContactEditWindowContent::from_contact(contact);
                                    }
                                    if ui.button("🗑").clicked() {
                                        self.edit_contact_mode =
                                            EditMode::Delete(public_key.clone());
                                    }
                                    ui.end_row();
                                },
                            );
                            match self.edit_contact_mode.clone() {
                                EditMode::Delete(delete_public_key) => {
                                    self.contacts.remove_contact(&delete_public_key);
                                    self.edit_contact_mode = EditMode::None;
                                }
                                _ => {}
                            }
                        });
                });
            });

        egui::SidePanel::left("edit_contacts")
            .min_width(60.)
            .show_animated(
                ctx,
                match self.edit_contact_mode {
                    EditMode::New => true,
                    EditMode::Edit(_) => true,
                    _ => false,
                },
                |ui| {
                    let title = match self.edit_contact_mode.clone() {
                        EditMode::New => "New Contact".to_string(),
                        EditMode::Edit(_) => {
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
                        .striped(false)
                        .show(ui, |ui| {
                            ui.label("Name:");
                            ui.add(
                                egui::TextEdit::singleline(
                                    &mut self.contact_edit_window_content.name,
                                )
                                .min_size(vec2(60., 0.)),
                            );
                            ui.end_row();

                            ui.label("Public Key:");
                            ui.text_edit_singleline(
                                &mut self.contact_edit_window_content.public_key,
                            );
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
                            .on_disabled_hover_text(
                                "Please add at least one participant and a name.",
                            )
                            .clicked()
                        {
                            match self.edit_contact_mode.clone() {
                                EditMode::New => {
                                    self.contacts.add_contact(Contact::from_contact_window(
                                        &self.contact_edit_window_content,
                                    ));
                                    self.edit_contact_mode = EditMode::None;
                                }
                                EditMode::Edit(public_key) => {
                                    self.contacts.remove_contact(&public_key);
                                    self.contacts.add_contact(Contact::from_contact_window(
                                        &self.contact_edit_window_content,
                                    ));

                                    self.edit_contact_mode = EditMode::None;
                                }
                                _ => {}
                            }
                        }
                    });
                },
            );

        egui::SidePanel::right("settigs_panel").show_animated(ctx, self.show_settings, |ui| {
            ui.heading("Settings");
            ui.separator();
            ScrollArea::vertical().show(ui, |ui| {
                ui.collapsing("Peers", |ui| {
                    Grid::new("peer_list_grid")
                        .num_columns(2)
                        .min_col_width(0.)
                        .show(ui, |ui| {
                            let mut peer_list_entry_edit_mode: EditMode<usize> = EditMode::None;
                            for (peer_index, peer) in self.settings.peers.iter().enumerate() {
                                if ui.button("🗑").on_hover_text("Remove peer").clicked() {
                                    peer_list_entry_edit_mode = EditMode::Delete(peer_index);
                                }
                                ui.add(Label::new(peer.to_string()).truncate(true));
                                ui.end_row();
                            }
                            if let EditMode::Delete(peer_index) = peer_list_entry_edit_mode {
                                self.settings.peers.remove(peer_index);
                            }
                        });
                    ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                        if ui
                            .add_enabled(
                                self.settings.current_peer_is_valid,
                                Button::new("➕ Add Peer"),
                            )
                            .on_disabled_hover_text("Please enter a valid Multi Address.")
                            .clicked()
                        {
                            match Multiaddr::from_str(&self.settings.current_peer) {
                                Ok(peer) => {
                                    self.settings.peers.push(peer.clone());
                                    self.settings.current_peer.clear();
                                    self.settings.current_peer_is_valid = false;
                                    if let Some(p2pc) = &mut self.p2pc {
                                        p2pc.execute(p2pc_lib::Action::Dial(peer)).ok();
                                    }
                                }
                                Err(_) => {
                                    log::warn!("Invalid peer address");
                                }
                            }
                        }
                        if ui
                            .add(TextEdit::singleline(&mut self.settings.current_peer))
                            .changed()
                        {
                            self.settings.current_peer_is_valid =
                                match Multiaddr::from_str(&self.settings.current_peer) {
                                    Ok(_) => true,
                                    Err(_) => false,
                                } && !self.settings.current_peer.is_empty();
                        }
                    });
                });

                ui.collapsing("Listen Addresses", |ui| {
                    for listen_address in self.listen_addresses.lock().unwrap().iter() {
                        ui.label(listen_address.to_string());
                        ui.separator();
                    }
                });
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_chat_id {
                Some(current_chat_id) => match self.chats.lock().unwrap().get_chat(&current_chat_id) {
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
                                    .striped(false)
                                    .show(ui, |ui| {
                                        for message in current_chat.get_chat_messages() {
                                            let sender = self.contacts.get_contact(message.get_sender());
                                            let sender_is_user =
                                                message.get_sender() == &own_public_key_base_64.clone();
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
                                                                        RichText::new(if message.get_sender().len() > 12 {
                                                                            format!(
                                                                                "{}...",
                                                                                &message.get_sender()[..12]
                                                                            )
                                                                        } else {
                                                                            message.get_sender().clone()
                                                                        })
                                                                        .color(egui::Color32::RED),
                                                                    ))
                                                                    .on_hover_text((&message.get_sender()).to_string());

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
                                                            self.current_message_answer_to = Some(*message.get_message_id());
                                                        }
                                                    });
                                                });
                                            });
                                            ui.end_row();
                                        }
                                    });
                            });
                    }
                    None => {
                        self.current_chat_id = None;
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
