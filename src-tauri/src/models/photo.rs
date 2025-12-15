use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PhotoMetadata {
    pub filename: String,
    pub full_path: String,
    pub filesize: u64,
    pub rating: u8,
    pub is_picked: bool,
}

#[cfg(test)]
mod tests {
    #[test]
    fn export_bindings() {
        // Sert à déclencher la génération grâce à #[ts(export)]
        // Le fichier sera créé dans le dossier "bindings/" à la racine du projet Rust.
    }
}
