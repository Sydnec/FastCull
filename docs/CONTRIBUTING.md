# Contribuer à FastCull

Merci de vouloir contribuer à FastCull ! Ce projet vise une performance absolue et une stabilité irréprochable. Voici nos standards.

## 📐 Architecture du Projet

Le projet est divisé en deux parties distinctes :

- **`src-tauri/` (Backend Rust) :** Gère l'accès disque, l'extraction binaire et la fenêtre système.
  - _Règle d'or :_ Sécurité mémoire et gestion d'erreurs explicite (pas de `unwrap()` sauvages).
- **`src/` (Frontend React) :** Gère l'interface utilisateur.
  - _Règle d'or :_ Fluidité. Pas de calculs lourds dans le thread JS.

## Workflow de Développement

1. **Fork & Branch :** Créez une branche pour votre feature (`feat/my-feature`) ou fix (`fix/crash-issue`).
2. **Commit :** Nous utilisons **Conventional Commits**.
   - ✅ `feat: add raw extraction logic`
   - ✅ `fix: resolve memory leak on large folders`
   - ❌ `Added stuff`, `WIP`
3. **Qualité (Automatisée) :**
   - Des **hooks Git (Husky)** sont actifs.
   - Ils lanceront automatiquement `eslint`, `prettier` et `cargo clippy` avant chaque commit.
   - Si le hook échoue, corrigez les erreurs avant de réessayer.

## 🦀 Standards Rust

- Le code doit passer `cargo fmt` et `cargo clippy` sans warnings.
- Privilégiez l'asynchronisme (`tokio`) pour toutes les opérations I/O.

## ⚛️ Standards React

- TypeScript strict activé. Pas de `any`.
- Composants fonctionnels et Hooks uniquement.

## 🐞 Signaler un Bug

Utilisez les templates d'issue fournis sur GitHub. Soyez précis sur le format RAW utilisé et l'OS.
