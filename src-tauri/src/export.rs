//! Module d'export : copie/déplacement des fichiers sélectionnés + génération XMP.

use crate::commands::AppState;
use crate::state::{ExportFilter, ExportMode, ExportResult, PickStatus};
use std::fs;
use std::io;
use std::path::Path;
use std::sync::Arc;

/// Exécute l'export selon le mode et le filtre choisis.
pub fn run_export(
    mode: ExportMode,
    filter: ExportFilter,
    state: &Arc<AppState>,
) -> Result<ExportResult, Box<dyn std::error::Error + Send + Sync>> {
    let files = state.files.read().unwrap();
    let folder = state
        .folder_path
        .read()
        .unwrap()
        .clone()
        .ok_or("Aucun dossier ouvert")?;

    // Collecter les fichiers correspondant au filtre
    let filtered: Vec<(usize, &crate::state::FileInfo)> = files
        .iter()
        .enumerate()
        .filter(|(i, _)| {
            let status = state
                .statuses
                .get(i)
                .map(|s| *s)
                .unwrap_or(PickStatus::Unrated);
            let rating = state.ratings.get(i).map(|r| *r).unwrap_or(0);

            let status_match = filter.statuses.contains(&status);
            let rating_match = rating >= filter.min_rating;

            status_match && rating_match
        })
        .collect();

    if filtered.is_empty() {
        return Ok(ExportResult {
            exported_count: 0,
            xmp_count: 0,
            output_dir: None,
        });
    }

    let mut exported_count = 0;
    let mut xmp_count = 0;
    let mut output_dir = None;

    match mode {
        ExportMode::Move | ExportMode::Copy => {
            // Créer le dossier Selected/
            let selected_dir = folder.join("Selected");
            fs::create_dir_all(&selected_dir)?;
            output_dir = Some(selected_dir.to_string_lossy().into());

            for (idx, file) in &filtered {
                let dest = selected_dir.join(&file.filename);

                match mode {
                    ExportMode::Move => {
                        fs::rename(&file.path, &dest).or_else(|_| {
                            // rename peut échouer entre disques, fallback sur copy + delete
                            fs::copy(&file.path, &dest)?;
                            fs::remove_file(&file.path)
                        })?;
                    }
                    ExportMode::Copy => {
                        fs::copy(&file.path, &dest)?;
                    }
                    _ => unreachable!(),
                }
                exported_count += 1;

                // Générer le XMP à côté du fichier exporté
                let status = state
                    .statuses
                    .get(idx)
                    .map(|s| *s)
                    .unwrap_or(PickStatus::Unrated);
                let rating = state.ratings.get(idx).map(|r| *r).unwrap_or(0);
                write_xmp(&dest, rating, status)?;
                xmp_count += 1;
            }
        }
        ExportMode::XmpOnly => {
            // Générer les XMP à côté des fichiers originaux
            for (idx, file) in &filtered {
                let status = state
                    .statuses
                    .get(idx)
                    .map(|s| *s)
                    .unwrap_or(PickStatus::Unrated);
                let rating = state.ratings.get(idx).map(|r| *r).unwrap_or(0);
                write_xmp(&file.path, rating, status)?;
                xmp_count += 1;
            }
        }
    }

    Ok(ExportResult {
        exported_count,
        xmp_count,
        output_dir,
    })
}

/// Génère un fichier XMP sidecar à côté du fichier RAW donné.
/// Le Label XMP correspond au statut : Pick → "Select", Reject → "Reject", Unrated → pas de Label.
fn write_xmp(raw_path: &Path, rating: u8, status: PickStatus) -> io::Result<()> {
    let xmp_path = raw_path.with_extension("xmp");

    let label_attr = match status {
        PickStatus::Pick => "\n      xmp:Label=\"Select\"",
        PickStatus::Reject => "\n      xmp:Label=\"Reject\"",
        PickStatus::Unrated => "",
    };

    let xmp_content = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
  <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
    <rdf:Description
      xmlns:xmp="http://ns.adobe.com/xap/1.0/"
      xmp:Rating="{}"{}  />
  </rdf:RDF>
</x:xmpmeta>
"#,
        rating, label_attr
    );

    fs::write(&xmp_path, xmp_content)
}
