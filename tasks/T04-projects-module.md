# T04 — Module Projets (CRUD + détection moteur)

**Statut** : TODO
**Dépendances** : T02 (App Shell), T03 (commandes Rust db_commands)

---

## Objectif

Implémenter la gestion complète des projets : créer, lister, ouvrir, supprimer. Inclut la détection automatique du moteur de jeu en Rust.

---

## Étapes

### 1. Commande Rust — detect_engine

Créer `src-tauri/src/commands/detect.rs` :

```rust
use std::path::Path;
use walkdir::WalkDir;

#[tauri::command]
pub fn detect_engine(game_path: String) -> Result<String, String> {
    let path = Path::new(&game_path);

    if !path.exists() {
        return Err("Dossier introuvable".into());
    }

    // RPG Maker MZ (data/ à la racine + System.json)
    if path.join("data").join("System.json").exists() {
        return Ok("rpgmz".into());
    }
    // RPG Maker MV (www/data/)
    if path.join("www").join("data").join("System.json").exists() {
        return Ok("rpgmv".into());
    }
    // RPG Maker VXAce
    if path.join("Data").join("Scripts.rvdata2").exists() {
        return Ok("rpgmvxa".into());
    }
    // RPG Maker VX
    if path.join("Data").join("Scripts.rvdata").exists() {
        return Ok("rpgmvx".into());
    }
    // RPG Maker XP
    if path.join("Data").join("Scripts.rxdata").exists() {
        return Ok("rpgmxp".into());
    }
    // Wolf RPG (GameDat.wolf dans BasicData/)
    if path.join("Data").join("BasicData").join("GameDat.wolf").exists()
        || path.join("Data").join("BasicData").join("GameDat.dat").exists()
    {
        return Ok("wolf".into());
    }
    // RPG Bakin (fichier .bakin à la racine)
    for entry in WalkDir::new(path).max_depth(1).into_iter().flatten() {
        if entry.path().extension().map(|e| e == "bakin").unwrap_or(false) {
            return Ok("bakin".into());
        }
    }

    Err("Moteur non reconnu. Vérifiez que c'est bien le dossier racine du jeu.".into())
}
```

Enregistrer dans `lib.rs` :
```rust
mod commands;
// ...
.invoke_handler(tauri::generate_handler![commands::detect::detect_engine])
```

### 2. Commandes Rust CRUD projets (T03 — db_commands.rs)

Les opérations CRUD projets sont définies dans `src-tauri/src/commands/db_commands.rs` (voir T03).
Commandes disponibles :

| Commande | Signature |
|----------|-----------|
| `list_projects` | `() → Result<Vec<Project>, String>` |
| `create_project` | `(name, game_path, work_path, engine, source_lang, target_lang, project_context) → Result<Project, String>` |
| `delete_project` | `(project_id: i64) → Result<(), String>` |

### 3. Vue — src/views/ProjectsView.vue

- Afficher la liste des projets via `useProjects()` composable (TanStack Vue Query)
- Afficher pour chaque projet : nom, moteur, barre de progression via `useProjectProgress(id)`
- Bouton "Nouveau projet" → ouvre un `Dialog`
- Bouton "Ouvrir" → `useOpenProject()(project)` → stocke dans `useProjectStore` + navigue vers `/projects/:id/translate`
- Bouton "Supprimer" → `AlertDialog` de confirmation → `useDeleteProject().mutate(id)`

### 4. Dialog — Nouveau projet (dans ProjectsView.vue)

```vue
<script setup lang="ts">
import { ref } from 'vue';
import { useRouter } from 'vue-router';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { useQueryClient } from '@tanstack/vue-query';

const router = useRouter();
const qc = useQueryClient();

async function handleCreate(formData: NewProjectForm) {
  // 1. Sélectionner le dossier du jeu via dialog natif
  const gameDir = await open({ directory: true, title: 'Dossier du jeu' });
  if (!gameDir) return;

  // 2. Détecter le moteur via Rust
  const engine = await invoke<string>('detect_engine', { gamePath: gameDir });

  // 3. Suggérer un dossier de travail
  const workPath = `${gameDir}_neoglot_work`;

  // 4. Créer le projet via Rust (rusqlite)
  const project = await invoke<Project>('create_project', {
    name: formData.name,
    gamePath: gameDir,
    workPath,
    engine,
    sourceLang: formData.sourceLang,
    targetLang: formData.targetLang,
    projectContext: formData.projectContext ?? '',
  });

  qc.invalidateQueries({ queryKey: ['projects'] });
  // 5. Naviguer vers la page de traduction
  router.push(`/projects/${project.id}/translate`);
}
</script>
```

### 5. Composable — src/composables/useProjects.ts

```typescript
import { invoke } from '@tauri-apps/api/core';
import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query';
import { useRouter } from 'vue-router';
import type { Project } from '@/types/project';
import { useProjectStore } from '@/stores';

export function useProjects() {
  return useQuery({
    queryKey: ['projects'],
    queryFn: () => invoke<Project[]>('list_projects'),
  });
}

export function useDeleteProject() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: number) => invoke('delete_project', { projectId: id }),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['projects'] }),
  });
}

// Ouvre un projet : stocke dans Pinia + navigue vers /translate
export function useOpenProject() {
  const router = useRouter();
  const projectStore = useProjectStore();

  return (project: Project) => {
    projectStore.setProject(project);
    router.push(`/projects/${project.id}/translate`);
  };
}
```

### 6. Progression dans la liste projets

Via `get_project_progress` (définie dans T03) :

```typescript
// Dans src/composables/useProjects.ts
export function useProjectProgress(projectId: number) {
  return useQuery({
    queryKey: ['project-progress', projectId],
    queryFn: () => invoke<{ done: number; total: number }>('get_project_progress', { projectId }),
    refetchInterval: 5_000, // rafraîchir pendant la traduction
  });
}
```

---

## Validation

- Créer un projet → détection moteur correcte pour un jeu RPG Maker MZ test
- La liste s'affiche avec progression
- Supprimer un projet → disparaît de la liste
- La barre de progression s'affiche (0% pour un nouveau projet)
