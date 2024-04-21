use std::collections::HashMap;
use super::Contact;

#[derive(serde::Deserialize, serde::Serialize)]
#[derive(Default)]
pub struct Contacts {
    contacts: HashMap<String, Contact>,
}



impl Contacts {
    pub fn add_contact(&mut self, contact: Contact) {
        self.contacts.insert(contact.public_key.clone(), contact);
    }

    pub fn get_contact(&self, public_key: &str) -> Option<&Contact> {
        self.contacts.get(public_key)
    }

    pub fn get_contacts(&self) -> &HashMap<String, Contact> {
        &self.contacts
    }

    pub fn remove_contact(&mut self, public_key: &str) {
        self.contacts.remove(public_key);
    }
}