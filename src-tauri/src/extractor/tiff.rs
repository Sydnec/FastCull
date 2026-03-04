//! Scanner TIFF/IFD pour l'extraction des JPEG encapsulés.
//!
//! Supporte les formats RAW basés sur TIFF : CR2, NEF, ARW, DNG, ORF, PEF, RW2, etc.
//!
//! Algorithme :
//! 1. Lire le header TIFF (byte order + magic + offset IFD0)
//! 2. Parcourir chaque IFD de la chaîne + SubIFDs récursivement
//! 3. Pour chaque IFD avec compression JPEG, collecter un candidat
//! 4. Filtrer : exclure les données capteur (NewSubFileType=0 + 1 sample/pixel)
//! 5. Parser les dimensions JPEG réelles si besoin
//! 6. Retourner le plus grand preview JPEG trouvé

use super::{ExtractionError, ExtractionResult};
use memmap2::Mmap;
use std::fs::File;
use std::path::Path;

// Tags TIFF pertinents
const TAG_NEW_SUBFILE_TYPE: u16 = 0x00FE;
const TAG_IMAGE_WIDTH: u16 = 0x0100;
const TAG_IMAGE_LENGTH: u16 = 0x0101;
const TAG_BITS_PER_SAMPLE: u16 = 0x0102;
const TAG_COMPRESSION: u16 = 0x0103;
const TAG_STRIP_OFFSETS: u16 = 0x0111;
const TAG_SAMPLES_PER_PIXEL: u16 = 0x0115;
const TAG_STRIP_BYTE_COUNTS: u16 = 0x0117;
const TAG_SUB_IFDS: u16 = 0x014A;
const TAG_JPEG_IF_OFFSET: u16 = 0x0201;
const TAG_JPEG_IF_BYTE_COUNT: u16 = 0x0202;

// Compression JPEG
const COMPRESSION_JPEG_OLD: u16 = 6;
const COMPRESSION_JPEG: u16 = 7;

/// Ordre des octets dans le fichier TIFF
#[derive(Debug, Clone, Copy)]
enum ByteOrder {
    LittleEndian,
    BigEndian,
}

impl ByteOrder {
    fn read_u16(self, data: &[u8], offset: usize) -> u16 {
        if offset + 2 > data.len() {
            return 0;
        }
        match self {
            ByteOrder::LittleEndian => u16::from_le_bytes([data[offset], data[offset + 1]]),
            ByteOrder::BigEndian => u16::from_be_bytes([data[offset], data[offset + 1]]),
        }
    }

    fn read_u32(self, data: &[u8], offset: usize) -> u32 {
        if offset + 4 > data.len() {
            return 0;
        }
        match self {
            ByteOrder::LittleEndian => u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]),
            ByteOrder::BigEndian => u32::from_be_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]),
        }
    }
}

/// Un candidat JPEG trouvé dans la structure TIFF
#[derive(Debug)]
struct JpegCandidate {
    offset: usize,
    length: usize,
    /// Dimensions depuis les tags TIFF
    tiff_width: u32,
    tiff_height: u32,
    /// Dimensions réelles parsées depuis le header JPEG
    jpeg_width: u32,
    jpeg_height: u32,
    /// NewSubFileType : 0 = full resolution (RAW), 1 = reduced (preview)
    new_subfile_type: Option<u32>,
    /// Nombre de samples par pixel (1 = RAW Bayer, 3 = RGB/JPEG preview)
    samples_per_pixel: u16,
    /// Bits par sample (14/12 = RAW, 8 = preview JPEG)
    bits_per_sample: u16,
}

impl JpegCandidate {
    /// Dimensions effectives (priorise JPEG réel > TIFF tags)
    fn effective_dimensions(&self) -> (u32, u32) {
        if self.jpeg_width > 0 && self.jpeg_height > 0 {
            (self.jpeg_width, self.jpeg_height)
        } else {
            (self.tiff_width, self.tiff_height)
        }
    }

    fn pixel_count(&self) -> u64 {
        let (w, h) = self.effective_dimensions();
        w as u64 * h as u64
    }

    /// Vérifie si ce candidat est probablement les données RAW capteur
    /// (et non un preview JPEG utilisable)
    fn is_likely_raw_data(&self) -> bool {
        // NewSubFileType = 0 signifie "full resolution image" = données capteur
        if self.new_subfile_type == Some(0) {
            return true;
        }
        // 1 sample/pixel + plus de 8 bits = données Bayer RAW compressées en JPEG
        if self.samples_per_pixel == 1 && self.bits_per_sample > 8 {
            return true;
        }
        false
    }
}

/// Extrait le plus grand JPEG preview d'un fichier TIFF-based.
pub fn extract_largest_jpeg(path: &Path) -> ExtractionResult {
    let file = File::open(path)?;
    let mmap = unsafe { Mmap::map(&file)? };
    let data = &mmap[..];

    if data.len() < 8 {
        return Err(ExtractionError::InvalidTiff("Fichier trop petit".into()));
    }

    // Lire le byte order
    let byte_order = match (data[0], data[1]) {
        (b'I', b'I') => ByteOrder::LittleEndian,
        (b'M', b'M') => ByteOrder::BigEndian,
        _ => return Err(ExtractionError::InvalidTiff("Byte order invalide".into())),
    };

    // Vérifier le magic number (42 pour TIFF standard)
    let magic = byte_order.read_u16(data, 2);
    if magic != 42 {
        return Err(ExtractionError::InvalidTiff(format!(
            "Magic number invalide : {} (attendu 42)",
            magic
        )));
    }

    // Offset du premier IFD
    let first_ifd_offset = byte_order.read_u32(data, 4) as usize;

    // Collecter tous les candidats JPEG
    let mut candidates: Vec<JpegCandidate> = Vec::new();
    scan_ifd_chain(data, byte_order, first_ifd_offset, &mut candidates, 0);

    // Compléter les dimensions JPEG réelles pour chaque candidat
    for c in &mut candidates {
        if c.offset + 2 <= data.len() && data[c.offset] == 0xFF && data[c.offset + 1] == 0xD8 {
            if let Some((w, h)) = parse_jpeg_dimensions(data, c.offset) {
                c.jpeg_width = w;
                c.jpeg_height = h;
            }
        }
    }

    // Stratégie de sélection :
    // 1. Préférer les candidats qui NE sont PAS des données RAW
    // 2. Parmi ceux-ci, prendre le plus grand par pixels
    // 3. Fallback : si aucun candidat "preview", prendre n'importe quel JPEG valide
    let preview_candidates: Vec<&JpegCandidate> = candidates
        .iter()
        .filter(|c| !c.is_likely_raw_data())
        .collect();

    let best = if !preview_candidates.is_empty() {
        preview_candidates
            .iter()
            .max_by_key(|c| c.pixel_count())
            .unwrap()
    } else {
        candidates
            .iter()
            .max_by_key(|c| c.pixel_count())
            .ok_or(ExtractionError::NoPreviewFound)?
    };

    // Vérifications de limites
    if best.offset >= data.len() || best.offset + best.length > data.len() {
        return Err(ExtractionError::InvalidTiff(
            "Offset JPEG hors limites".into(),
        ));
    }

    // Vérifier le marqueur SOI
    if data[best.offset] != 0xFF || data[best.offset + 1] != 0xD8 {
        return Err(ExtractionError::NoPreviewFound);
    }

    Ok(data[best.offset..best.offset + best.length].to_vec())
}

/// Parcourt une chaîne d'IFDs et collecte les candidats JPEG.
fn scan_ifd_chain(
    data: &[u8],
    bo: ByteOrder,
    mut ifd_offset: usize,
    candidates: &mut Vec<JpegCandidate>,
    depth: u32,
) {
    if depth > 10 {
        return;
    }

    while ifd_offset > 0 && ifd_offset + 2 < data.len() {
        let entry_count = bo.read_u16(data, ifd_offset) as usize;
        let entries_start = ifd_offset + 2;

        let mut tiff_width: u32 = 0;
        let mut tiff_height: u32 = 0;
        let mut compression: u16 = 0;
        let mut strip_offset: u32 = 0;
        let mut strip_byte_count: u32 = 0;
        let mut jpeg_offset: u32 = 0;
        let mut jpeg_byte_count: u32 = 0;
        let mut sub_ifd_offsets: Vec<usize> = Vec::new();
        let mut new_subfile_type: Option<u32> = None;
        let mut samples_per_pixel: u16 = 0;
        let mut bits_per_sample: u16 = 0;

        for i in 0..entry_count {
            let entry_offset = entries_start + i * 12;
            if entry_offset + 12 > data.len() {
                break;
            }

            let tag = bo.read_u16(data, entry_offset);
            let typ = bo.read_u16(data, entry_offset + 2);
            let count = bo.read_u32(data, entry_offset + 4) as usize;
            let value_offset = entry_offset + 8;

            match tag {
                TAG_NEW_SUBFILE_TYPE => {
                    new_subfile_type = Some(read_value(data, bo, typ, value_offset));
                }
                TAG_IMAGE_WIDTH => {
                    tiff_width = read_value(data, bo, typ, value_offset);
                }
                TAG_IMAGE_LENGTH => {
                    tiff_height = read_value(data, bo, typ, value_offset);
                }
                TAG_BITS_PER_SAMPLE => {
                    bits_per_sample = bo.read_u16(data, value_offset);
                }
                TAG_COMPRESSION => {
                    compression = bo.read_u16(data, value_offset);
                }
                TAG_SAMPLES_PER_PIXEL => {
                    samples_per_pixel = bo.read_u16(data, value_offset);
                }
                TAG_STRIP_OFFSETS => {
                    strip_offset = read_value(data, bo, typ, value_offset);
                }
                TAG_STRIP_BYTE_COUNTS => {
                    strip_byte_count = read_value(data, bo, typ, value_offset);
                }
                TAG_JPEG_IF_OFFSET => {
                    jpeg_offset = read_value(data, bo, typ, value_offset);
                }
                TAG_JPEG_IF_BYTE_COUNT => {
                    jpeg_byte_count = read_value(data, bo, typ, value_offset);
                }
                TAG_SUB_IFDS => {
                    if count == 1 {
                        sub_ifd_offsets.push(read_value(data, bo, typ, value_offset) as usize);
                    } else {
                        let offsets_ptr = bo.read_u32(data, value_offset) as usize;
                        for j in 0..count {
                            let off = bo.read_u32(data, offsets_ptr + j * 4) as usize;
                            if off > 0 {
                                sub_ifd_offsets.push(off);
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // Candidat via StripOffsets (compression JPEG)
        if (compression == COMPRESSION_JPEG || compression == COMPRESSION_JPEG_OLD)
            && strip_offset > 0
            && strip_byte_count > 0
        {
            candidates.push(JpegCandidate {
                offset: strip_offset as usize,
                length: strip_byte_count as usize,
                tiff_width,
                tiff_height,
                jpeg_width: 0,
                jpeg_height: 0,
                new_subfile_type,
                samples_per_pixel,
                bits_per_sample,
            });
        }

        // Candidat via JPEGInterchangeFormat
        if jpeg_offset > 0 && jpeg_byte_count > 0 {
            candidates.push(JpegCandidate {
                offset: jpeg_offset as usize,
                length: jpeg_byte_count as usize,
                tiff_width,
                tiff_height,
                jpeg_width: 0,
                jpeg_height: 0,
                new_subfile_type,
                samples_per_pixel,
                bits_per_sample,
            });
        }

        // Parcourir les SubIFDs récursivement
        for sub_offset in sub_ifd_offsets {
            scan_ifd_chain(data, bo, sub_offset, candidates, depth + 1);
        }

        // Passer à l'IFD suivant
        let next_ifd_pos = entries_start + entry_count * 12;
        if next_ifd_pos + 4 > data.len() {
            break;
        }
        ifd_offset = bo.read_u32(data, next_ifd_pos) as usize;
    }
}

/// Parse les dimensions réelles d'un JPEG en scannant les marqueurs SOF.
fn parse_jpeg_dimensions(data: &[u8], offset: usize) -> Option<(u32, u32)> {
    let mut pos = offset + 2; // skip SOI marker
    let max = (offset + 65536).min(data.len());

    while pos + 4 < max {
        if data[pos] != 0xFF {
            pos += 1;
            continue;
        }

        let marker = data[pos + 1];

        // SOF markers (C0-CF sauf C4, C8, CC)
        if (0xC0..=0xCF).contains(&marker) && marker != 0xC4 && marker != 0xC8 && marker != 0xCC {
            if pos + 9 < data.len() {
                let height = u16::from_be_bytes([data[pos + 5], data[pos + 6]]) as u32;
                let width = u16::from_be_bytes([data[pos + 7], data[pos + 8]]) as u32;
                return Some((width, height));
            }
        }

        // Skip marker segment
        if marker == 0x00 || marker == 0xFF || (0xD0..=0xD9).contains(&marker) {
            pos += 1;
            continue;
        }

        if pos + 3 < data.len() {
            let seg_len = u16::from_be_bytes([data[pos + 2], data[pos + 3]]) as usize;
            pos += 2 + seg_len;
        } else {
            break;
        }
    }
    None
}

/// Lit une valeur depuis un entry TIFF selon son type.
fn read_value(data: &[u8], bo: ByteOrder, typ: u16, offset: usize) -> u32 {
    match typ {
        1 => data.get(offset).copied().unwrap_or(0) as u32, // BYTE
        3 => bo.read_u16(data, offset) as u32,               // SHORT
        4 => bo.read_u32(data, offset),                      // LONG
        _ => bo.read_u32(data, offset),                      // Fallback
    }
}
