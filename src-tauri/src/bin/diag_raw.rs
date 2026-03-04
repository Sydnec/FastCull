//! Diagnostic détaillé de la structure TIFF/IFD des fichiers RAW.
//!
//! Usage: cargo run --bin diag_raw

use memmap2::Mmap;
use std::fs::File;
use std::path::Path;

fn main() {
    let samples_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("samples");

    let sample_files = [
        ("Sony ARW", "sample.ARW"),
        ("Canon CR2", "sample.cr2"),
        ("Nikon NEF", "sample.NEF"),
    ];

    for (label, filename) in &sample_files {
        let path = samples_dir.join(filename);
        if !path.exists() {
            continue;
        }
        println!("\n{}", "=".repeat(60));
        println!("=== {} ({}) ===", label, filename);
        println!("{}\n", "=".repeat(60));
        analyze_file(&path);
    }
}

fn analyze_file(path: &Path) {
    let file = File::open(path).unwrap();
    let mmap = unsafe { Mmap::map(&file).unwrap() };
    let data = &mmap[..];

    println!("Taille fichier: {:.2} Mo", data.len() as f64 / 1_048_576.0);

    // Byte order
    let bo = match (data[0], data[1]) {
        (b'I', b'I') => {
            println!("Byte order: Little Endian (Intel)");
            ByteOrder::LE
        }
        (b'M', b'M') => {
            println!("Byte order: Big Endian (Motorola)");
            ByteOrder::BE
        }
        _ => {
            println!("Byte order inconnu: {:02X} {:02X}", data[0], data[1]);
            return;
        }
    };

    let magic = bo.u16(data, 2);
    println!("Magic: {} (attendu: 42)", magic);

    let first_ifd = bo.u32(data, 4) as usize;
    println!("Premier IFD offset: {}\n", first_ifd);

    scan_ifd(data, bo, first_ifd, 0, 0);
}

fn scan_ifd(data: &[u8], bo: ByteOrder, mut offset: usize, depth: u32, mut ifd_num: u32) {
    if depth > 8 || offset == 0 || offset + 2 >= data.len() {
        return;
    }

    while offset > 0 && offset + 2 < data.len() {
        let indent = "  ".repeat(depth as usize);
        let count = bo.u16(data, offset) as usize;
        println!(
            "{}IFD#{} @ offset {} ({} entries)",
            indent, ifd_num, offset, count
        );

        let entries_start = offset + 2;

        let mut width: u32 = 0;
        let mut height: u32 = 0;
        let mut compression: u16 = 0;
        let mut new_subfile_type: u32 = 0;
        let mut strip_offset: u32 = 0;
        let mut strip_count: u32 = 0;
        let mut jpeg_offset: u32 = 0;
        let mut jpeg_count: u32 = 0;
        let mut sub_ifds: Vec<usize> = Vec::new();
        let mut bits_per_sample: u16 = 0;
        let mut samples_per_pixel: u16 = 0;

        for i in 0..count {
            let eo = entries_start + i * 12;
            if eo + 12 > data.len() {
                break;
            }

            let tag = bo.u16(data, eo);
            let typ = bo.u16(data, eo + 2);
            let cnt = bo.u32(data, eo + 4) as usize;
            let vo = eo + 8;

            match tag {
                0x00FE => {
                    new_subfile_type = read_val(data, bo, typ, vo);
                    println!(
                        "{}  Tag 0x00FE NewSubFileType = {} {}",
                        indent,
                        new_subfile_type,
                        match new_subfile_type {
                            0 => "(full resolution)",
                            1 => "(reduced resolution / preview)",
                            2 => "(single page of multi-page)",
                            _ => "(other)",
                        }
                    );
                }
                0x0100 => {
                    width = read_val(data, bo, typ, vo);
                    println!("{}  Tag 0x0100 ImageWidth = {}", indent, width);
                }
                0x0101 => {
                    height = read_val(data, bo, typ, vo);
                    println!("{}  Tag 0x0101 ImageLength = {}", indent, height);
                }
                0x0102 => {
                    bits_per_sample = bo.u16(data, vo);
                    println!("{}  Tag 0x0102 BitsPerSample = {}", indent, bits_per_sample);
                }
                0x0103 => {
                    compression = bo.u16(data, vo);
                    println!(
                        "{}  Tag 0x0103 Compression = {} {}",
                        indent,
                        compression,
                        match compression {
                            1 => "(uncompressed)",
                            6 => "(JPEG old)",
                            7 => "(JPEG)",
                            8 => "(deflate)",
                            34713 => "(Nikon NEF compressed)",
                            65535 => "(Pentax PEF)",
                            _ => "(other)",
                        }
                    );
                }
                0x0111 => {
                    strip_offset = read_val(data, bo, typ, vo);
                    println!(
                        "{}  Tag 0x0111 StripOffsets = {} (count={})",
                        indent, strip_offset, cnt
                    );
                }
                0x0115 => {
                    samples_per_pixel = bo.u16(data, vo);
                    println!(
                        "{}  Tag 0x0115 SamplesPerPixel = {}",
                        indent, samples_per_pixel
                    );
                }
                0x0117 => {
                    strip_count = read_val(data, bo, typ, vo);
                    println!(
                        "{}  Tag 0x0117 StripByteCounts = {} ({:.2} Mo) (count={})",
                        indent,
                        strip_count,
                        strip_count as f64 / 1_048_576.0,
                        cnt
                    );
                }
                0x014A => {
                    println!("{}  Tag 0x014A SubIFDs (count={})", indent, cnt);
                    if cnt == 1 {
                        let off = read_val(data, bo, typ, vo) as usize;
                        sub_ifds.push(off);
                        println!("{}    -> SubIFD @ {}", indent, off);
                    } else {
                        let ptr = bo.u32(data, vo) as usize;
                        for j in 0..cnt {
                            let off = bo.u32(data, ptr + j * 4) as usize;
                            if off > 0 {
                                sub_ifds.push(off);
                                println!("{}    -> SubIFD @ {}", indent, off);
                            }
                        }
                    }
                }
                0x0201 => {
                    jpeg_offset = read_val(data, bo, typ, vo);
                    println!("{}  Tag 0x0201 JPEGInterchangeFormat = {}", indent, jpeg_offset);
                }
                0x0202 => {
                    jpeg_count = read_val(data, bo, typ, vo);
                    println!(
                        "{}  Tag 0x0202 JPEGInterchangeFormatLength = {} ({:.2} Mo)",
                        indent,
                        jpeg_count,
                        jpeg_count as f64 / 1_048_576.0
                    );
                }
                0x8769 => {
                    let exif_off = read_val(data, bo, typ, vo) as usize;
                    println!("{}  Tag 0x8769 ExifIFD @ {}", indent, exif_off);
                }
                _ => {}
            }
        }

        // Résumé pour cet IFD
        let is_jpeg = compression == 6 || compression == 7;
        if width > 0 || height > 0 {
            println!(
                "{}  => Dimensions: {}x{}, Compression: {}, JPEG: {}",
                indent, width, height, compression, is_jpeg
            );
        }

        // Vérifier les données aux offsets trouvés
        if strip_offset > 0 && (strip_offset as usize) + 2 <= data.len() {
            let b0 = data[strip_offset as usize];
            let b1 = data[strip_offset as usize + 1];
            let is_soi = b0 == 0xFF && b1 == 0xD8;
            println!(
                "{}  => Strip bytes: [{:02X} {:02X}] {}",
                indent,
                b0,
                b1,
                if is_soi { "JPEG SOI OK" } else { "NOT JPEG" }
            );
            if is_soi && strip_count > 0 {
                // Parse JPEG dimensions
                if let Some((jw, jh)) = parse_jpeg_dimensions(data, strip_offset as usize) {
                    println!(
                        "{}  => JPEG real dimensions: {}x{} ({:.2} Mo)",
                        indent,
                        jw,
                        jh,
                        strip_count as f64 / 1_048_576.0
                    );
                }
            }
        }

        if jpeg_offset > 0 && (jpeg_offset as usize) + 2 <= data.len() {
            let b0 = data[jpeg_offset as usize];
            let b1 = data[jpeg_offset as usize + 1];
            let is_soi = b0 == 0xFF && b1 == 0xD8;
            println!(
                "{}  => JPEG IF bytes: [{:02X} {:02X}] {}",
                indent,
                b0,
                b1,
                if is_soi {
                    "JPEG SOI OK"
                } else {
                    "NOT JPEG"
                }
            );
            if is_soi && jpeg_count > 0 {
                if let Some((jw, jh)) = parse_jpeg_dimensions(data, jpeg_offset as usize) {
                    println!(
                        "{}  => JPEG IF real dimensions: {}x{} ({:.2} Mo)",
                        indent,
                        jw,
                        jh,
                        jpeg_count as f64 / 1_048_576.0
                    );
                }
            }
        }

        println!();

        // Recurse in SubIFDs
        for (si, sub_off) in sub_ifds.iter().enumerate() {
            scan_ifd(data, bo, *sub_off, depth + 1, si as u32);
        }

        // Next IFD
        let next_pos = entries_start + count * 12;
        if next_pos + 4 > data.len() {
            break;
        }
        offset = bo.u32(data, next_pos) as usize;
        ifd_num += 1;
    }
}

/// Parse les dimensions réelles d'un JPEG en scannant les marqueurs SOF
fn parse_jpeg_dimensions(data: &[u8], offset: usize) -> Option<(u32, u32)> {
    let mut pos = offset + 2; // skip SOI
    let max = (offset + 65536).min(data.len()); // limiter la recherche

    while pos + 4 < max {
        if data[pos] != 0xFF {
            pos += 1;
            continue;
        }

        let marker = data[pos + 1];

        // SOF markers (C0-CF except C4, C8, CC)
        if (0xC0..=0xCF).contains(&marker) && marker != 0xC4 && marker != 0xC8 && marker != 0xCC
        {
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

#[derive(Clone, Copy)]
enum ByteOrder {
    LE,
    BE,
}

impl ByteOrder {
    fn u16(self, d: &[u8], o: usize) -> u16 {
        if o + 2 > d.len() {
            return 0;
        }
        match self {
            Self::LE => u16::from_le_bytes([d[o], d[o + 1]]),
            Self::BE => u16::from_be_bytes([d[o], d[o + 1]]),
        }
    }
    fn u32(self, d: &[u8], o: usize) -> u32 {
        if o + 4 > d.len() {
            return 0;
        }
        match self {
            Self::LE => u32::from_le_bytes([d[o], d[o + 1], d[o + 2], d[o + 3]]),
            Self::BE => u32::from_be_bytes([d[o], d[o + 1], d[o + 2], d[o + 3]]),
        }
    }
}

fn read_val(data: &[u8], bo: ByteOrder, typ: u16, offset: usize) -> u32 {
    match typ {
        1 => data.get(offset).copied().unwrap_or(0) as u32,
        3 => bo.u16(data, offset) as u32,
        4 => bo.u32(data, offset),
        _ => bo.u32(data, offset),
    }
}
