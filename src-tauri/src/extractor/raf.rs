//! Extracteur de JPEG pour les fichiers Fuji RAF.
//!
//! Le format RAF a un header propriétaire avec l'offset et la taille
//! du JPEG embarqué à des positions fixes.

use super::{ExtractionError, ExtractionResult};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

/// Taille minimale d'un header RAF
const RAF_HEADER_SIZE: usize = 92;

/// Extrait le JPEG encapsulé d'un fichier Fuji RAF.
///
/// Structure du header RAF :
/// - Octets 0-15 : Magic "FUJIFILMCCD-RAW "
/// - Octets 84-87 : Offset du JPEG (big-endian u32)
/// - Octets 88-91 : Taille du JPEG (big-endian u32)
pub fn extract_jpeg(path: &Path) -> ExtractionResult {
    let mut file = File::open(path)?;

    // Lire le header
    let mut header = [0u8; RAF_HEADER_SIZE];
    file.read_exact(&mut header)?;

    // Vérifier le magic
    if &header[0..16] != b"FUJIFILMCCD-RAW " {
        return Err(ExtractionError::InvalidTiff(
            "Header RAF invalide (magic number incorrect)".into(),
        ));
    }

    // Lire l'offset et la taille du JPEG (big-endian)
    let jpeg_offset = u32::from_be_bytes([header[84], header[85], header[86], header[87]]) as u64;
    let jpeg_length = u32::from_be_bytes([header[88], header[89], header[90], header[91]]) as usize;

    if jpeg_length == 0 {
        return Err(ExtractionError::NoPreviewFound);
    }

    // Seek vers le JPEG et le lire
    file.seek(SeekFrom::Start(jpeg_offset))?;
    let mut jpeg_data = vec![0u8; jpeg_length];
    file.read_exact(&mut jpeg_data)?;

    // Vérifier que c'est bien un JPEG (SOI marker)
    if jpeg_data.len() < 2 || jpeg_data[0] != 0xFF || jpeg_data[1] != 0xD8 {
        return Err(ExtractionError::NoPreviewFound);
    }

    Ok(jpeg_data)
}
