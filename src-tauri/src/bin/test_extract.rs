//! Test d'extraction des JPEG encapsulés dans les fichiers RAW sample.
//!
//! Usage: cargo run --bin test_extract

use std::path::Path;
use std::time::Instant;

fn main() {
    let samples_dir = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("samples");

    let sample_files = [
        ("Sony ARW", "sample.ARW"),
        ("Canon CR2", "sample.cr2"),
        ("Nikon NEF", "sample.NEF"),
    ];

    println!("=== FastCull - Test d'extraction JPEG ===\n");
    println!("Dossier samples : {}\n", samples_dir.display());

    let output_dir = samples_dir.join("_extracted");
    std::fs::create_dir_all(&output_dir).ok();

    let mut all_ok = true;

    for (label, filename) in &sample_files {
        let path = samples_dir.join(filename);

        if !path.exists() {
            println!("[SKIP] {} - fichier introuvable: {}", label, path.display());
            continue;
        }

        let file_size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        print!("[TEST] {} ({}, {:.1} Mo) ... ", label, filename, file_size as f64 / 1_048_576.0);

        let start = Instant::now();
        match fastcull_lib::extractor::extract_preview(&path) {
            Ok(jpeg_data) => {
                let elapsed = start.elapsed();
                let jpeg_size = jpeg_data.len();

                // Vérifier que c'est un JPEG valide (SOI marker)
                let valid_jpeg = jpeg_data.len() >= 2
                    && jpeg_data[0] == 0xFF
                    && jpeg_data[1] == 0xD8;

                if valid_jpeg {
                    println!(
                        "OK ({:.1} ms, JPEG: {:.1} Mo, ratio: {:.0}%)",
                        elapsed.as_secs_f64() * 1000.0,
                        jpeg_size as f64 / 1_048_576.0,
                        (jpeg_size as f64 / file_size as f64) * 100.0,
                    );

                    // Écrire le JPEG extrait pour validation visuelle
                    let out_path = output_dir.join(format!("{}.jpg", filename));
                    if let Err(e) = std::fs::write(&out_path, &jpeg_data) {
                        println!("  [WARN] Impossible d'écrire {}: {}", out_path.display(), e);
                    } else {
                        println!("  -> Sauvegardé: {}", out_path.display());
                    }
                } else {
                    println!("ERREUR - données JPEG invalides (pas de marqueur SOI)");
                    all_ok = false;
                }
            }
            Err(e) => {
                let elapsed = start.elapsed();
                println!("ERREUR ({:.1} ms): {}", elapsed.as_secs_f64() * 1000.0, e);
                all_ok = false;
            }
        }
    }

    // Test de performance : extractions multiples
    println!("\n=== Test de performance (10 extractions par fichier) ===\n");

    for (label, filename) in &sample_files {
        let path = samples_dir.join(filename);
        if !path.exists() {
            continue;
        }

        let mut times = Vec::new();
        for _ in 0..10 {
            let start = Instant::now();
            let _ = fastcull_lib::extractor::extract_preview(&path);
            times.push(start.elapsed().as_secs_f64() * 1000.0);
        }

        let avg = times.iter().sum::<f64>() / times.len() as f64;
        let min = times.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = times.iter().cloned().fold(0.0_f64, f64::max);

        println!(
            "{:12} : moy={:.1}ms  min={:.1}ms  max={:.1}ms  {}",
            label,
            avg,
            min,
            max,
            if avg < 50.0 { "PASS" } else { "SLOW" }
        );
    }

    println!("\n=== Résultat : {} ===", if all_ok { "SUCCES" } else { "ECHECS DETECTES" });

    if !all_ok {
        std::process::exit(1);
    }
}
