//! Extracteur de JPEG encapsulés dans les fichiers RAW.
//!
//! Stratégie : parcourir la structure TIFF/IFD pour localiser le plus grand
//! aperçu JPEG sans jamais décoder les données capteur brutes.

pub mod tiff;
pub mod raf;

use std::path::Path;

/// Résultat de l'extraction : les octets bruts du JPEG encapsulé
pub type ExtractionResult = Result<Vec<u8>, ExtractionError>;

#[derive(Debug, thiserror::Error)]
pub enum ExtractionError {
    #[error("Format de fichier non supporté : {0}")]
    UnsupportedFormat(String),
    #[error("Aucun aperçu JPEG trouvé dans le fichier")]
    NoPreviewFound,
    #[error("Erreur d'I/O : {0}")]
    Io(#[from] std::io::Error),
    #[error("Structure TIFF invalide : {0}")]
    InvalidTiff(String),
}

/// Extrait le plus grand JPEG encapsulé d'un fichier RAW.
///
/// Dispatche vers le bon extracteur selon l'extension du fichier.
pub fn extract_preview(path: &Path) -> ExtractionResult {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        // Formats TIFF-based : CR2, CR3, NEF, ARW, DNG, ORF, PEF, RW2, SRW, 3FR
        "cr2" | "cr3" | "nef" | "arw" | "dng" | "orf" | "pef" | "rw2" | "srw" | "3fr" => {
            tiff::extract_largest_jpeg(path)
        }
        // Fuji RAF : format propriétaire avec offset JPEG dans le header
        "raf" => raf::extract_jpeg(path),
        _ => Err(ExtractionError::UnsupportedFormat(ext)),
    }
}
