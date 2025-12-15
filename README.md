# ⚡ FastCull

> **Le dérushage photo instantané, sans chargement, sans frustration.**

FastCull est une application de bureau (Windows, macOS, Linux) conçue pour résoudre le goulot d'étranglement des photographes : le tri (culling) des fichiers RAW.
Contrairement aux éditeurs classiques qui tentent de développer le RAW, FastCull extrait binairement la preview JPEG intégrée pour une performance **0-latency**.

## 🚀 Fonctionnalités Clés

- **Performance Native :** Moteur Rust pour une gestion I/O imbattable.
- **Zero-Latency :** Affichage instantané des RAWs (ARW, CR3, NEF).
- **Workflow "Game-ifié" :** Navigation clavier optimisée pour le tri rapide.
- **Non-destructif :** Génération de fichiers sidecar `.XMP` compatibles Lightroom/Capture One.
- **Privacy First :** 100% Local. Aucune donnée cloud.

## 🛠 Tech Stack

- **Core :** [Tauri](https://tauri.app/) (Rust)
- **Frontend :** React + TypeScript + Vite
- **Styling :** TailwindCSS / CSS Modules
- **Quality :** ESLint, Prettier, Husky, Commitlint

## 🏗 Installation & Développement

### Pré-requis
- Node.js (v18+)
- Rust (v1.70+)
- Outils de build natifs (Visual Studio C++ Build Tools sur Windows, Xcode Command Line Tools sur Mac)

### Lancer le projet

```bash
# 1. Installer les dépendances
npm install

# 2. Lancer en mode développement (Hot Reload)
npm run tauri dev
```

### Build pour production

```bash
npm run tauri build
```

## 🗺 Roadmap

- [ ] **Phase 1 (Moteur) :** Extraction binaire des previews JPEG via Rust.
- [ ] **Phase 2 (UI) :** Interface React fluide et navigation clavier.
- [ ] **Phase 3 (Data) :** Écriture des fichiers XMP standards.
- [ ] **Phase 4 (Packaging) :** Installateurs .exe et .dmg.

## 📄 Licence

Ce projet est sous licence MIT. Voir le fichier [LICENSE](LICENSE) pour plus de détails.
