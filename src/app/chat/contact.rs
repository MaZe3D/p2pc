use random_color::{Luminosity, RandomColor};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Contact {
    pub public_key: String,
    pub name: String,
    pub color: egui::Color32,
}

impl Default for Contact {
    fn default() -> Self {
        Self {
            public_key: String::new(),
            name: String::new(),
            color: egui::Color32::from_rgb(0, 0, 0),
        }
    }
}

impl Contact {
    pub fn from_contact_window(contact_window: &ContactEditWindowContent) -> Self {
        Self {
            public_key: contact_window.public_key.clone(),
            name: contact_window.name.clone(),
            color: egui::Color32::from_rgba_premultiplied(
                (contact_window.color[0] * 255.) as u8,
                (contact_window.color[1] * 255.) as u8,
                (contact_window.color[2] * 255.) as u8,
                255,
            ),
        }
    }
}
pub struct ContactEditWindowContent {
    pub public_key: String,
    pub name: String,
    pub color: [f32; 3],
}

impl Default for ContactEditWindowContent {
    fn default() -> Self {
        Self {
            public_key: "".to_string(),
            name: "".to_string(),
            color: (|| {
                let rgb_color = RandomColor::new()
                    .luminosity(Luminosity::Bright) // Optional
                    .seed(rand::random::<i32>()) // Optional
                    .alpha(1.0) // Optional
                    .to_rgb_array();
                [
                    rgb_color[0] as f32 / 255.,
                    rgb_color[1] as f32 / 255.,
                    rgb_color[2] as f32 / 255.,
                ]
            })(),
        }
    }
}

impl ContactEditWindowContent {
    pub fn from_contact(contact: &Contact) -> Self {
        Self {
            public_key: contact.public_key.clone(),
            name: contact.name.clone(),
            color: [
                contact.color.r() as f32 / 255.,
                contact.color.g() as f32 / 255.,
                contact.color.b() as f32 / 255.,
            ],
        }
    }
}


