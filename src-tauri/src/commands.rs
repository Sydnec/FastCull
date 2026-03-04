//! Commandes Tauri exposées au frontend via `invoke`.

use crate::cache::ImageCache;
use crate::state::{is_raw_extension, DirEntry, DriveInfo, ExportFilter, ExportMode, ExportResult, FileInfo, PickStatus};
use dashmap::DashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};
use walkdir::WalkDir;

/// État global de l'application, partagé entre les commandes Tauri
pub struct AppState {
    pub files: RwLock<Vec<FileInfo>>,
    pub current_index: AtomicUsize,
    pub statuses: DashMap<usize, PickStatus>,
    pub ratings: DashMap<usize, u8>,
    pub cache: ImageCache,
    pub folder_path: RwLock<Option<PathBuf>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            files: RwLock::new(Vec::new()),
            current_index: AtomicUsize::new(0),
            statuses: DashMap::new(),
            ratings: DashMap::new(),
            cache: ImageCache::new(),
            folder_path: RwLock::new(None),
        }
    }
}

/// Ouvre une liste de chemins (fichiers RAW et/ou dossiers).
/// Les dossiers sont scannés récursivement pour trouver les fichiers RAW.
#[tauri::command]
pub fn open_paths(
    paths: Vec<String>,
    state: tauri::State<'_, Arc<AppState>>,
    app: tauri::AppHandle,
) -> Result<Vec<FileInfo>, String> {
    let mut files: Vec<FileInfo> = Vec::new();
    let mut base_folder: Option<PathBuf> = None;
    let mut seen_paths = std::collections::HashSet::new();

    for path_str in &paths {
        let path = PathBuf::from(path_str);

        if path.is_dir() {
            if base_folder.is_none() {
                base_folder = Some(path.clone());
            }
            // Scan récursif du dossier (pas de max_depth)
            for entry in WalkDir::new(&path).into_iter().filter_map(|e| e.ok()) {
                let file_path = entry.path().to_path_buf();
                if !file_path.is_file() {
                    continue;
                }
                if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
                    if is_raw_extension(ext) {
                        let canonical = file_path.to_string_lossy().to_string();
                        if seen_paths.insert(canonical) {
                            let metadata = std::fs::metadata(&file_path).ok();
                            files.push(FileInfo {
                                index: 0,
                                filename: file_path
                                    .file_name()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .into(),
                                size: metadata.map(|m| m.len()).unwrap_or(0),
                                path: file_path,
                                status: PickStatus::Unrated,
                                rating: 0,
                            });
                        }
                    }
                }
            }
        } else if path.is_file() {
            if base_folder.is_none() {
                base_folder = path.parent().map(|p| p.to_path_buf());
            }
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if is_raw_extension(ext) {
                    let canonical = path.to_string_lossy().to_string();
                    if seen_paths.insert(canonical) {
                        let metadata = std::fs::metadata(&path).ok();
                        files.push(FileInfo {
                            index: 0,
                            filename: path
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .into(),
                            size: metadata.map(|m| m.len()).unwrap_or(0),
                            path: path.clone(),
                            status: PickStatus::Unrated,
                            rating: 0,
                        });
                    }
                }
            }
        }
    }

    if files.is_empty() {
        return Err("Aucun fichier RAW trouvé dans les chemins sélectionnés".into());
    }

    // Trier par nom de fichier
    files.sort_by(|a, b| a.filename.cmp(&b.filename));

    // Réindexer après tri
    for (i, file) in files.iter_mut().enumerate() {
        file.index = i;
    }

    // Mettre à jour l'état global
    *state.folder_path.write().unwrap() = base_folder;
    *state.files.write().unwrap() = files.clone();
    state.current_index.store(0, Ordering::Relaxed);
    state.statuses.clear();
    state.ratings.clear();
    state.cache.clear();

    // Pré-charger les premières photos (asynchrone, non-bloquant)
    let files_ref = state.files.read().unwrap();
    state.cache.update_window_async(0, &files_ref, app);

    Ok(files.clone())
}

/// Change la photo courante et met à jour la fenêtre de prefetch.
/// Retourne immédiatement — le prefetch tourne en arrière-plan.
#[tauri::command]
pub fn navigate(
    index: usize,
    state: tauri::State<'_, Arc<AppState>>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let files = state.files.read().unwrap();
    if index >= files.len() {
        return Err(format!(
            "Index {} hors limites (max {})",
            index,
            files.len()
        ));
    }

    state.current_index.store(index, Ordering::Relaxed);
    state.cache.update_window_async(index, &files, app);

    Ok(())
}

/// Définit le statut Pick/Reject d'une photo.
#[tauri::command]
pub fn set_pick_status(
    index: usize,
    status: PickStatus,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let files = state.files.read().unwrap();
    if index >= files.len() {
        return Err(format!("Index {} hors limites", index));
    }
    state.statuses.insert(index, status);
    Ok(())
}

/// Définit la note (1-5) d'une photo.
#[tauri::command]
pub fn set_rating(
    index: usize,
    rating: u8,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<(), String> {
    if rating > 5 {
        return Err("La note doit être entre 0 et 5".into());
    }
    let files = state.files.read().unwrap();
    if index >= files.len() {
        return Err(format!("Index {} hors limites", index));
    }
    state.ratings.insert(index, rating);
    Ok(())
}

/// Récupère le JPEG d'une photo depuis le cache (ou l'extrait à la volée).
#[tauri::command]
pub fn get_image(
    index: usize,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<Vec<u8>, String> {
    // Essayer le cache d'abord
    if let Some(jpeg) = state.cache.get(index) {
        return Ok((*jpeg).clone());
    }

    // Sinon extraire à la volée
    let files = state.files.read().unwrap();
    if index >= files.len() {
        return Err(format!("Index {} hors limites", index));
    }

    let path = &files[index].path;
    let jpeg = crate::extractor::extract_preview(path).map_err(|e| e.to_string())?;

    // Mettre en cache pour les prochains accès
    state.cache.insert(index, jpeg.clone());

    Ok(jpeg)
}

/// Exporte les photos selon le mode et le filtre choisis.
#[tauri::command]
pub fn export_selected(
    mode: ExportMode,
    filter: ExportFilter,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<ExportResult, String> {
    crate::export::run_export(mode, filter, &state).map_err(|e| e.to_string())
}

/// Liste les lecteurs disponibles (Windows).
#[tauri::command]
pub fn list_drives() -> Result<Vec<DriveInfo>, String> {
    let mut drives = Vec::new();

    #[cfg(target_os = "windows")]
    {
        for letter in b'A'..=b'Z' {
            let drive_path = format!("{}:\\", letter as char);
            let path = PathBuf::from(&drive_path);
            if path.exists() {
                drives.push(DriveInfo {
                    letter: format!("{}:", letter as char),
                    label: String::new(),
                    total_bytes: 0,
                });
            }
        }
    }

    Ok(drives)
}

/// Liste le contenu d'un répertoire (dossiers + fichiers RAW).
/// Si `recursive` est vrai, le `raw_count` de chaque sous-dossier est récursif.
#[tauri::command]
pub fn list_directory(path: String, recursive: bool) -> Result<Vec<DirEntry>, String> {
    let dir = PathBuf::from(&path);
    if !dir.is_dir() {
        return Err(format!("{} n'est pas un répertoire", path));
    }

    let read_dir = std::fs::read_dir(&dir).map_err(|e| e.to_string())?;
    let mut result: Vec<DirEntry> = Vec::new();

    for entry in read_dir.filter_map(|e| e.ok()) {
        let file_type = match entry.file_type() {
            Ok(ft) => ft,
            Err(_) => continue,
        };
        let name = entry.file_name().to_string_lossy().into_owned();
        let full_path = entry.path();

        if file_type.is_dir() {
            let raw_count = if recursive {
                count_raw_recursive(&full_path)
            } else {
                count_raw_shallow(&full_path)
            };
            result.push(DirEntry {
                name,
                path: full_path.to_string_lossy().into_owned(),
                is_dir: true,
                size: 0,
                raw_count,
            });
        } else if file_type.is_file() {
            if let Some(ext) = full_path.extension().and_then(|e| e.to_str()) {
                if is_raw_extension(ext) {
                    let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                    result.push(DirEntry {
                        name,
                        path: full_path.to_string_lossy().into_owned(),
                        is_dir: false,
                        size,
                        raw_count: 0,
                    });
                }
            }
        }
    }

    // Tri : dossiers d'abord, puis fichiers, alphabétique
    result.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });

    Ok(result)
}

/// Compte les fichiers RAW au premier niveau d'un dossier.
fn count_raw_shallow(dir: &PathBuf) -> usize {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
                .filter(|e| {
                    e.path()
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .map(|ext| is_raw_extension(ext))
                        .unwrap_or(false)
                })
                .count()
        })
        .unwrap_or(0)
}

/// Compte les fichiers RAW récursivement dans un dossier.
fn count_raw_recursive(dir: &PathBuf) -> usize {
    WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| is_raw_extension(ext))
                .unwrap_or(false)
        })
        .count()
}

/// Ouvre un chemin dans l'Explorateur Windows.
#[tauri::command]
pub fn open_in_explorer(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
