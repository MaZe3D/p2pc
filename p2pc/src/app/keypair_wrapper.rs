pub struct Keypair(pub libp2p::identity::Keypair);

impl serde::Serialize for Keypair {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(
            &self
                .0
                .to_protobuf_encoding()
                .map_err(|e| serde::ser::Error::custom(format!("{}", e)))?,
        )
    }
}

impl<'de> serde::Deserialize<'de> for Keypair {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Keypair(
            libp2p::identity::Keypair::from_protobuf_encoding(<&[u8]>::deserialize(deserializer)?)
                .map_err(|e| serde::de::Error::custom(format!("{}", e)))?,
        ))
    }
}
