//! Cache de prefetch pour les aperçus JPEG.
//!
//! Maintient une fenêtre glissante de 14 photos en RAM (N-5 à N+8).
//! Utilise DashMap pour un accès concurrent sans locks.
//! Le prefetch est asynchrone (rayon::spawn) pour ne jamais bloquer la navigation.

use dashmap::DashMap;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::Emitter;

use crate::extractor;
use crate::state::FileInfo;

/// Nombre de photos à pré-charger en avant
const PREFETCH_FORWARD: usize = 8;
/// Nombre de photos à pré-charger en arrière
const PREFETCH_BACKWARD: usize = 5;

/// Payload de l'événement prefetch_progress
#[derive(Clone, Serialize)]
struct PrefetchProgress {
    cached: Vec<usize>,
    total: usize,
}

/// Cache d'images JPEG en mémoire
pub struct ImageCache {
    /// Données JPEG indexées par position dans la liste (Arc pour zero-copy)
    data: Arc<DashMap<usize, Arc<Vec<u8>>>>,
}

impl ImageCache {
    pub fn new() -> Self {
        Self {
            data: Arc::new(DashMap::new()),
        }
    }

    /// Récupère un JPEG du cache. Retourne None si absent.
    /// Utilise Arc pour éviter de cloner les données JPEG (zero-copy).
    pub fn get(&self, index: usize) -> Option<Arc<Vec<u8>>> {
        self.data.get(&index).map(|entry| Arc::clone(entry.value()))
    }

    /// Vérifie si un index est en cache.
    pub fn contains(&self, index: usize) -> bool {
        self.data.contains_key(&index)
    }

    /// Insère un JPEG dans le cache.
    pub fn insert(&self, index: usize, jpeg: Vec<u8>) {
        self.data.insert(index, Arc::new(jpeg));
    }

    /// Met à jour la fenêtre de cache de manière asynchrone (non-bloquant).
    ///
    /// 1. Calcule la nouvelle fenêtre [index - BACKWARD, index + FORWARD]
    /// 2. Évince les entrées hors fenêtre (synchrone, rapide)
    /// 3. Lance l'extraction des entrées manquantes en arrière-plan via rayon::spawn
    /// 4. Émet un événement `prefetch_progress` au frontend après chaque extraction
    pub fn update_window_async(
        &self,
        current_index: usize,
        files: &[FileInfo],
        app_handle: tauri::AppHandle,
    ) {
        let total = files.len();
        if total == 0 {
            return;
        }

        // Calculer les bornes de la fenêtre
        let start = current_index.saturating_sub(PREFETCH_BACKWARD);
        let end = (current_index + PREFETCH_FORWARD).min(total - 1);

        // Éviction synchrone : supprimer les entrées hors fenêtre
        let keys_to_remove: Vec<usize> = self
            .data
            .iter()
            .map(|entry| *entry.key())
            .filter(|&k| k < start || k > end)
            .collect();

        for key in keys_to_remove {
            self.data.remove(&key);
        }

        // Identifier les indices manquants dans la fenêtre
        let mut missing: Vec<(usize, PathBuf)> = (start..=end)
            .filter(|i| !self.data.contains_key(i))
            .map(|i| (i, files[i].path.clone()))
            .collect();

        if missing.is_empty() {
            // Tout est en cache, notifier le frontend
            let _ = app_handle.emit(
                "prefetch_progress",
                PrefetchProgress {
                    cached: self.cached_indices(),
                    total: end - start + 1,
                },
            );
            return;
        }

        // Prioriser : photo courante d'abord, puis forward, puis backward
        missing.sort_by_key(|(i, _)| {
            if *i == current_index {
                0
            } else if *i > current_index {
                *i - current_index
            } else {
                current_index - *i + PREFETCH_FORWARD
            }
        });

        let cache = Arc::clone(&self.data);
        let window_size = end - start + 1;

        // Lancer chaque extraction en arrière-plan (non-bloquant)
        for (index, path) in missing {
            let cache = cache.clone();
            let app_handle = app_handle.clone();
            rayon::spawn(move || {
                // Skip si déjà extrait entre-temps (navigation rapide)
                if cache.contains_key(&index) {
                    return;
                }
                match extractor::extract_preview(&path) {
                    Ok(jpeg) => {
                        cache.insert(index, Arc::new(jpeg));
                        let cached: Vec<usize> = cache.iter().map(|e| *e.key()).collect();
                        let _ = app_handle.emit(
                            "prefetch_progress",
                            PrefetchProgress {
                                cached,
                                total: window_size,
                            },
                        );
                        log::debug!("Cache: extrait index {} ({:?})", index, path.file_name());
                    }
                    Err(e) => {
                        log::warn!("Cache: échec extraction index {} : {}", index, e);
                    }
                }
            });
        }
    }

    /// Liste des indices actuellement en cache.
    pub fn cached_indices(&self) -> Vec<usize> {
        self.data.iter().map(|entry| *entry.key()).collect()
    }

    /// Vide entièrement le cache.
    pub fn clear(&self) {
        self.data.clear();
    }
}
