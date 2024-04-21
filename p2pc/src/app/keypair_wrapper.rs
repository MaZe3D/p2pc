use base64::Engine;

pub struct Keypair(pub libp2p::identity::Keypair);

impl serde::Serialize for Keypair {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0
            .to_protobuf_encoding()
            .map_err(|e| serde::ser::Error::custom(format!("{}", e)))?
            .serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Keypair {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Keypair(
            libp2p::identity::Keypair::from_protobuf_encoding(&<Vec<u8>>::deserialize(
                deserializer,
            )?)
            .map_err(|e| serde::de::Error::custom(format!("{}", e)))?,
        ))
    }
}

impl Default for Keypair {
    fn default() -> Self {
        Keypair(libp2p::identity::Keypair::generate_ed25519())
    }
}

impl Keypair {
    pub fn get_public_key_base64(&self) -> String {
        base64::engine::general_purpose::STANDARD.encode(self.0.public().encode_protobuf())
    }
}
