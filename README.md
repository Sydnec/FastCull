# FastCull

Application de tri photo ultra-rapide pour fichiers RAW. Conçue pour les photographes qui veulent trier rapidement leurs prises de vue avec un workflow clavier, des notes et un export XMP compatible Lightroom.

## Fonctionnalités

- **Navigation rapide** au clavier entre les photos RAW
- **Pick / Reject / Unrated** pour marquer les photos
- **Notes 0-5 étoiles** cliquables et au clavier
- **Vue simple** avec filmstrip horizontal (style Lightroom)
- **Vue mosaïque** pour sélection/notation de masse
- **Filtres** par statut et note minimale, appliqués à la navigation
- **Navigateur de fichiers** intégré avec comptage RAW récursif
- **Drag & drop** de dossiers
- **Export** : copie, déplacement ou XMP sidecar uniquement
- **XMP Lightroom** : `xmp:Rating` + `xmp:Label` (Select/Reject)

## Formats RAW supportés

Canon CR2/CR3, Nikon NEF, Sony ARW, Adobe DNG, Olympus ORF, Pentax PEF, Panasonic RW2, Fujifilm RAF, Samsung SRW, Hasselblad 3FR.

## Raccourcis clavier

| Touche | Action |
|---|---|
| `←` `↑` | Photo précédente |
| `→` `↓` | Photo suivante |
| `D` | Retenue (Pick) |
| `Q` | Rejetée (Reject) |
| `S` | Non notée (Unrated) |
| `1`-`5` | Note étoiles |
| `0` | Effacer la note |
| `Tab` | Basculer vue simple / mosaïque |
| `E` | Ouvrir l'export |
| `Échap` | Fermer le dialogue |

## Stack technique

| | |
|---|---|
| Framework | Tauri v2 |
| Frontend | React 19, TypeScript, Vite |
| Backend | Rust |
| Cache | DashMap (lock-free), fenêtre glissante de 14 images |
| Prefetch | rayon (thread pool asynchrone) |
| Formats | Parsing TIFF/IFD natif, extraction JPEG embedded |

## Architecture

```
src/                          # Frontend React
├── App.tsx                   # Welcome screen + drag-drop
├── hooks/
│   ├── useCull.ts            # État central (fichiers, statuts, filtres)
│   └── useKeyboard.ts        # Raccourcis clavier
├── components/
│   ├── Viewer.tsx            # Vue simple + mosaïque
│   ├── Filmstrip.tsx         # Bande de miniatures (lazy loading)
│   ├── GridView.tsx          # Grille de miniatures
│   ├── FilterBar.tsx         # Filtres statut + note
│   ├── StatusBar.tsx         # Barre d'info + contrôles
│   ├── ExportDialog.tsx      # Dialogue d'export
│   └── FileBrowser.tsx       # Navigateur de fichiers
└── styles/globals.css

src-tauri/src/                # Backend Rust
├── lib.rs                    # Setup Tauri + protocole preview://
├── commands.rs               # Commandes Tauri (open, navigate, export...)
├── cache.rs                  # Cache glissant Arc<Vec<u8>> + prefetch rayon
├── export.rs                 # Export fichiers + génération XMP
├── state.rs                  # Types (FileInfo, PickStatus, formats RAW)
└── extractor/
    ├── mod.rs                # Dispatch par format
    ├── tiff.rs               # Parser TIFF/IFD (CR2, NEF, ARW, DNG...)
    └── raf.rs                # Parser Fujifilm RAF
```

### Pipeline d'images

1. L'utilisateur importe un dossier → scan récursif via `walkdir`
2. Le backend extrait les JPEG embedded des RAW (jamais de décodage raw)
3. Cache glissant de 14 images (N-5 à N+8) via `DashMap<usize, Arc<Vec<u8>>>`
4. Prefetch asynchrone avec `rayon::spawn`, priorisé : courant → avant → arrière
5. Protocole custom `preview://` sert les JPEG au WebView avec `Cache-Control: immutable`
6. Frontend : `img.decode()` non-bloquant avec annulation des chargements obsolètes

## Développement

### Prérequis

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://rustup.rs/) stable
- [Tauri CLI](https://v2.tauri.app/start/prerequisites/)

### Lancer en développement

```bash
npm install
npm run tauri dev
```

### Build de production

```bash
npm run tauri build
```

## Licence

MIT
