# T01 — Installer les plugins Tauri + shadcn-vue + Tailwind

**Statut** : DONE
**Dépendances** : aucune (première tâche)
**Durée estimée** : setup complet

---

## Objectif

Partir du scaffold Tauri v2 de base et ajouter :
1. Tous les plugins Tauri P0/P1 nécessaires
2. shadcn/ui + Tailwind CSS v4
3. Vue Router pour la navigation
4. TanStack Vue Query pour les fetches

---

## Étapes

### 1. Installer les plugins Tauri (frontend)

```bash
pnpm add @tauri-apps/plugin-shell \
         @tauri-apps/plugin-fs \
         @tauri-apps/plugin-dialog \
         @tauri-apps/plugin-store \
         @tauri-apps/plugin-log \
         @tauri-apps/plugin-window-state \
         @tauri-apps/plugin-notification \
         @tauri-apps/plugin-process \
         @tauri-apps/plugin-clipboard-manager
```

**Ne pas installer** :
- `@tauri-apps/plugin-sql` — SQLite géré uniquement par `rusqlite` côté Rust (évite deux drivers SQLite sur le même fichier)
- `@tauri-apps/plugin-http` — appels Ollama via `reqwest` côté Rust (blocking, pas de streaming)

### 2. Ajouter les plugins Tauri (Cargo.toml)

Modifier `src-tauri/Cargo.toml` :

```toml
[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-shell = "2"
tauri-plugin-fs = "2"
tauri-plugin-dialog = "2"
tauri-plugin-store = "2"
tauri-plugin-log = "2"
tauri-plugin-window-state = "2"
tauri-plugin-notification = "2"
tauri-plugin-process = "2"
tauri-plugin-clipboard-manager = "2"

# Logique métier (pas de Python — tout en Rust)
# Pas de traduction temps réel — appels Ollama batch (POST /api/generate, stream: false)
reqwest = { version = "0.13", features = ["json", "blocking"] }
rusqlite = { version = "0.38", features = ["bundled"] }
tokio = { version = "1", features = ["full"] }
regex = "1"
once_cell = "1"

# Parsing RPG Maker (à ajouter en T04/T05 avec les bonnes versions crates.io)
# rvpacker-txt-rs-lib, rpgmad-lib, marshal-rs

# Utilitaires
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sha2 = "0.10"
log = "^0.4"
walkdir = "2"
```

Retirer `tauri-plugin-http` (les appels Ollama passent par `reqwest` blocking côté Rust).
Retirer `tauri-plugin-opener` (non utilisé dans NeoGlot).
Ne pas utiliser `ollama-rs` — dépend de `gxhash` qui nécessite AES+SSE2 CPU.

### 3. Enregistrer les plugins dans src-tauri/src/lib.rs

```rust
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::LogDir {
                        file_name: Some("neoglot".into()),
                    }),
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout),
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Webview),
                ])
                .level(log::LevelFilter::Info)
                .build()
        )
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![])
        .run(tauri::generate_context!())
        .expect("Erreur démarrage NeoGlot");
}
```

### 4. Mettre à jour capabilities/default.json

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Permissions NeoGlot",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "shell:allow-execute",
    "shell:allow-spawn",
    "shell:allow-kill",
    "fs:allow-read-recursive",
    "fs:allow-write-recursive",
    "fs:allow-exists",
    "fs:allow-mkdir",
    "fs:allow-copy-file",
    "dialog:default",
    "store:default",
    "window-state:default",
    "notification:default",
    "process:default",
    "clipboard-manager:default"
  ]
}
```

### 5. Configurer les commandes shell autorisées (tauri.conf.json)

Ajouter dans `tauri.conf.json` :

```json
{
  "plugins": {
    "shell": {
      "open": false
    }
  }
}
```

Les commandes spécifiques (wolftl, uberwolf) seront ajoutées à T09.

### 6. Installer Vue 3 + Tailwind + shadcn-vue

```bash
# Tailwind CSS v4
pnpm add tailwindcss @tailwindcss/vite

# Plugin Vite Vue
pnpm add -D @vitejs/plugin-vue

# shadcn-vue init (génère components.json + configure Tailwind + alias @/)
pnpm dlx shadcn-vue@latest init
```

Répondre au prompt shadcn-vue init :
- Style : **New York**
- Base color : **Zinc**
- CSS variables : **Yes**
- TypeScript : **Yes**

Le fichier `components.json` généré :
```json
{
  "$schema": "https://shadcn-vue.com/schema.json",
  "style": "new-york",
  "typescript": true,
  "tailwind": {
    "css": "src/style.css",
    "baseColor": "zinc",
    "cssVariables": true
  },
  "aliases": {
    "components": "@/components",
    "composables": "@/composables",
    "utils": "@/lib/utils",
    "ui": "@/components/ui",
    "lib": "@/lib"
  },
  "iconLibrary": "lucide"
}
```

Mettre à jour `vite.config.ts` :

```typescript
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import tailwindcss from '@tailwindcss/vite'
import { resolve } from 'path'

export default defineConfig({
  plugins: [vue(), tailwindcss()],
  resolve: {
    alias: { '@': resolve(__dirname, 'src') },
  },
})
```

Ajouter la classe `.dark` sur `<html>` dans `index.html` pour le thème sombre par défaut.

### 7. Installer les composants shadcn-vue nécessaires

```bash
pnpm dlx shadcn-vue@latest add button card badge progress dialog \
  input select textarea separator table scroll-area \
  alert-dialog resizable sidebar sonner spinner
```

### 8. Installer Vue Router + TanStack Vue Query

```bash
pnpm add vue-router @tanstack/vue-query
```

### 9. Vérifier le build

```bash
pnpm tauri dev
```

---

## Fichiers modifiés

- `src-tauri/Cargo.toml` — ajout dépendances Rust
- `src-tauri/src/lib.rs` — enregistrement plugins
- `src-tauri/capabilities/default.json` — permissions
- `src-tauri/tauri.conf.json` — config shell
- `package.json` — dépendances frontend
- `vite.config.ts` — plugins Vue + Tailwind + alias `@/`
- `src/style.css` — import Tailwind + thème shadcn-vue
- `components.json` — config shadcn-vue
- `index.html` — `class="dark"` sur `<html>`

## Validation

- `pnpm tauri dev` démarre sans erreur
- La page par défaut s'affiche avec le style shadcn-vue/Tailwind sombre
- Pas d'erreur de permissions dans la console DevTools
