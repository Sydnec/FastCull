mod cache;
mod commands;
mod export;
pub mod extractor;
mod state;

use commands::AppState;
use std::sync::Arc;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = Arc::new(AppState::new());

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .manage(app_state.clone())
        .register_uri_scheme_protocol("preview", move |_ctx, request| {
            // Protocole custom : preview://localhost/{index}?t={timestamp}
            // Le paramètre ?t= est un cache-buster pour forcer le rechargement WebView
            let uri = request.uri().to_string();

            // Extraire l'index depuis l'URI, en ignorant les query params
            let path_part = uri.split('?').next().unwrap_or(&uri);
            let index: usize = path_part
                .split('/')
                .last()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);

            let state = &app_state;

            // Essayer le cache
            if let Some(jpeg) = state.cache.get(index) {
                return tauri::http::Response::builder()
                    .status(200)
                    .header("Content-Type", "image/jpeg")
                    .header("Cache-Control", "max-age=3600, immutable")
                    .header("Access-Control-Allow-Origin", "*")
                    .body((*jpeg).clone())
                    .unwrap();
            }

            // Sinon extraire à la volée
            let files = state.files.read().unwrap();
            if index < files.len() {
                if let Ok(jpeg) = extractor::extract_preview(&files[index].path) {
                    state.cache.insert(index, jpeg.clone());
                    return tauri::http::Response::builder()
                        .status(200)
                        .header("Content-Type", "image/jpeg")
                        .header("Cache-Control", "max-age=3600, immutable")
                        .header("Access-Control-Allow-Origin", "*")
                        .body(jpeg)
                        .unwrap();
                }
            }

            // Erreur : image non trouvée
            tauri::http::Response::builder()
                .status(404)
                .header("Access-Control-Allow-Origin", "*")
                .body(Vec::new())
                .unwrap()
        })
        .invoke_handler(tauri::generate_handler![
            commands::open_paths,
            commands::navigate,
            commands::set_pick_status,
            commands::set_rating,
            commands::get_image,
            commands::export_selected,
            commands::list_drives,
            commands::list_directory,
            commands::open_in_explorer,
        ])
        .run(tauri::generate_context!())
        .expect("Erreur au lancement de l'application");
}
