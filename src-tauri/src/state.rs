use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Statut de tri d'une photo
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PickStatus {
    Unrated,
    Pick,
    Reject,
}

impl Default for PickStatus {
    fn default() -> Self {
        Self::Unrated
    }
}

/// Informations sur un fichier RAW
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub index: usize,
    pub filename: String,
    #[serde(skip)]
    pub path: PathBuf,
    pub size: u64,
    pub status: PickStatus,
    pub rating: u8,
}

/// Mode d'export des photos sélectionnées
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportMode {
    Move,
    Copy,
    XmpOnly,
}

impl Default for ExportMode {
    fn default() -> Self {
        Self::Move
    }
}

/// Filtre d'export (quelles photos exporter)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportFilter {
    /// Statuts à inclure (ex: ["pick", "reject"])
    pub statuses: Vec<PickStatus>,
    /// Note minimale (0 = pas de filtre)
    pub min_rating: u8,
}

/// Résultat de l'export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResult {
    pub exported_count: usize,
    pub xmp_count: usize,
    pub output_dir: Option<String>,
}

/// Information sur un lecteur (navigateur de fichiers)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveInfo {
    pub letter: String,
    pub label: String,
    pub total_bytes: u64,
}

/// Entrée de répertoire dans le navigateur de fichiers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub raw_count: usize,
}

/// Extensions RAW supportées
pub const RAW_EXTENSIONS: &[&str] = &[
    "cr2", "cr3", "nef", "arw", "dng", "orf", "pef", "rw2", "raf", "srw", "3fr",
];

/// Vérifie si une extension est un fichier RAW supporté
pub fn is_raw_extension(ext: &str) -> bool {
    RAW_EXTENSIONS.contains(&ext.to_lowercase().as_str())
}
