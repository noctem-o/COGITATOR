use anyhow::Result;
use blake3::Hasher;

pub struct Witness {
    hash: [u8; 32],
}

impl Witness {
    pub fn new(metadata_bytes: &[u8]) -> Result<Self> {
        let mut hasher = Hasher::new();
        hasher.update(b"COGITATOR/WITNESS/V1/INIT");
        hasher.update(metadata_bytes);
        let hash = *hasher.finalize().as_bytes();
        Ok(Self { hash })
    }

    pub fn update(&mut self, event_bytes: &[u8]) -> Result<()> {
        let mut hasher = Hasher::new();
        hasher.update(b"COGITATOR/WITNESS/V1/STEP");
        hasher.update(&self.hash);
        hasher.update(&(event_bytes.len() as u64).to_be_bytes());
        hasher.update(event_bytes);
        self.hash = *hasher.finalize().as_bytes();
        Ok(())
    }

    pub fn finalize_hex(&self) -> String {
        blake3::Hash::from(self.hash).to_hex().to_string()
    }
}
