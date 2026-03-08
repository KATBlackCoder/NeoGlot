# Structure UI — NeoGlot

## Stack UI

- **Vue 3** + TypeScript + Composition API (`<script setup lang="ts">`)
- **shadcn-vue** (Reka UI primitives) + **Tailwind CSS v4**
- Thème : **sombre** par défaut (dark mode natif shadcn-vue)
- Layout : navigation latérale fixe + zone de contenu principale

---

## Navigation principale

```
┌─────────────────────────────────────────────────────┐
│  NeoGlot                           [_] [□] [×]      │
├──────────┬──────────────────────────────────────────┤
│          │                                          │
│  🏠 Home │          Zone de contenu                 │
│          │                                          │
│  📁 Proj.│                                          │
│          │                                          │
│  🔤 Trans│                                          │
│          │                                          │
│  📚 Gloss│                                          │
│          │                                          │
│  ⚙️ Sett.│                                          │
│          │                                          │
└──────────┴──────────────────────────────────────────┘
```

**Composant** : `AppShell.vue` avec `AppSidebar.vue` (shadcn-vue) + `<main>` content area.

---

## Pages

### 1. Home (`/`)

Écran d'accueil affiché au démarrage.

```
┌─────────────────────────────────────────────────────┐
│                                                     │
│   NeoGlot                                           │
│   Traducteur de jeux RPG open-source                │
│                                                     │
│  ┌─────────────────────────────────────────────┐   │
│  │  ✅ Ollama connecté — llama3:8b             │   │  ← Badge vert/rouge
│  └─────────────────────────────────────────────┘   │
│                                                     │
│  Projets récents :                                  │
│  ┌─────────────┐  ┌─────────────┐                  │
│  │ Jeu A       │  │ Jeu B       │                  │
│  │ RPG Maker MZ│  │ Wolf RPG    │                  │
│  │ 43% traduit │  │ 12% traduit │                  │
│  └─────────────┘  └─────────────┘                  │
│                                                     │
│  [+ Nouveau projet]                                 │
│                                                     │
└─────────────────────────────────────────────────────┘
```

**Composants shadcn-vue** : `Card`, `Badge`, `Button`
**Données** : `useProjects()` composable → `invoke('list_projects')`

---

### 2. Projects (`/projects`)

Liste et gestion des projets.

```
┌──────────────────────────────────────────────────────┐
│  Projets               [+ Nouveau projet]            │
│  ────────────────────────────────────────────────── │
│                                                      │
│  ┌──────────────────────────────────────────────┐   │
│  │ Jeu A                   RPG Maker MZ         │   │
│  │ /home/user/games/jeu-a                       │   │
│  │ ████████████░░░░░░░░ 43% (1234/2872 strings) │   │  ← Progress
│  │ Dernier: 2026-03-07      [Ouvrir] [Supprimer]│   │
│  └──────────────────────────────────────────────┘   │
│                                                      │
│  ┌──────────────────────────────────────────────┐   │
│  │ Jeu B                   Wolf RPG             │   │
│  │ /home/user/games/jeu-b                       │   │
│  │ ██░░░░░░░░░░░░░░░░░░ 12%  (456/3800 strings) │   │
│  │ Dernier: 2026-03-05      [Ouvrir] [Supprimer]│   │
│  └──────────────────────────────────────────────┘   │
│                                                      │
└──────────────────────────────────────────────────────┘
```

**Dialog — Nouveau projet** :
```
Nom du projet    [________________]
Dossier du jeu   [________________] [Parcourir...]
Dossier travail  [________________] [Parcourir...]  ← auto-suggéré
Moteur détecté   [RPG Maker MZ   ] (auto-détection)
Langue source    [Japonais    ▼]
Langue cible     [Français    ▼]
Contexte projet  [_________________________________]  ← pour l'IA

                 [Annuler]  [Créer projet]
```

**Composants shadcn-vue** : `Card`, `Progress`, `Dialog`, `Input`, `Select`, `Textarea`, `Button`

---

### 3. Translate (`/projects/:id/translate`)

Vue principale de traduction pour un projet.

```
┌──────────────────────────────────────────────────────────┐
│  ← Jeu A                               Modèle: [llama3:8b ▼] │
│  RPG Maker MZ — 43% traduit                              │
│  ──────────────────────────────────────────────────────  │
│                                                          │
│  Fichiers         │ Strings                              │
│  ────────────     │ ────────────────────────────────     │
│  ▼ data/          │ [Tous ▼] [Status: Tous ▼] [🔍]      │
│    Map001 ██ 78%  │                                      │
│    Map002 ░░ 0%   │ ┌──────────────────────────────┐    │
│    Map003 ██ 90%  │ │ #1  【源氏】                  │    │
│  ▶ www/           │ │ Original : ありがとう          │    │
│                   │ │ Traduit  : Merci              │    │
│                   │ │ ✅ traduit                    │    │
│                   │ └──────────────────────────────┘    │
│                   │ ┌──────────────────────────────┐    │
│                   │ │ #2                           │    │
│                   │ │ Original : これは試練だ       │    │
│                   │ │ Traduit  : —                 │    │
│                   │ │ ⏳ en attente                │    │
│                   │ └──────────────────────────────┘    │
│                   │                                      │
│  [▶ Traduire tout]  [⏸ Pause]  [Exporter]               │
│  ████████████░░░░░ 43%  1234/2872  ~18 min restantes     │
└──────────────────────────────────────────────────────────┘
```

**Composants shadcn-vue** : `ResizablePanelGroup`, `ScrollArea`, `Badge`, `Progress`, `Button`, `Select`, `Input`, `AlertDialog` (overlay extraction)
**Extraction** : `AlertDialog` modal bloquant pendant l'extraction (barre de progression, fichier en cours, empêche navigation) ; `await nextTick()` pour garantir le rendu avant lancement ; toast succès/erreur via `vue-sonner`
**Streaming** : `Channel<TranslationProgress>` de `@tauri-apps/api/core` via `useTranslation()` composable

---

### 4. Glossary (`/projects/:id/glossary`)

Gestion du glossaire par projet.

```
┌──────────────────────────────────────────────────────┐
│  Glossaire — Jeu A              [+ Ajouter terme]    │
│  ──────────────────────────────────────────────────  │
│  🔍 [Rechercher terme...      ]                      │
│                                                      │
│  Terme         Traduction    Note       Mode  Action │
│  ─────────── ─────────────  ───────── ─────  ──────  │
│  勇者          Héros          Perso. pr. exact  [✏️][🗑] │
│  魔王          Démon-Roi      Boss final  exact  [✏️][🗑] │
│  \n[1]         Yuki           Héro, fém.  exact  [✏️][🗑] │
│                                                      │
│  Glossaire global (partagé entre tous les projets)  │
│  ─────────────────────────────────────────────────   │
│  お前           toi (fam.)     Pronom    contains [✏️][🗑] │
│                                                      │
└──────────────────────────────────────────────────────┘
```

**Composants shadcn-vue** : `Table`, `Input`, `Badge`, `Button`, `Dialog`

---

### 5. Settings (`/settings`)

Préférences de l'application.

```
┌──────────────────────────────────────────────────┐
│  Paramètres                                      │
│  ──────────────────────────────────────────────  │
│                                                  │
│  Ollama                                          │
│  URL        [http://localhost:11434         ]    │
│  Modèle     [llama3:8b                    ▼]    │
│             [Tester la connexion]                │
│                                                  │
│  Traduction                                      │
│  Tokens/batch  [2048   ]                         │
│  Langue source [Japonais  ▼]                     │
│  Langue cible  [Français  ▼]                     │
│                                                  │
│  Interface                                       │
│  Thème      [Sombre ▼]                           │
│  Langue UI  [Français ▼]                         │
│                                                  │
│  [Sauvegarder]                                   │
└──────────────────────────────────────────────────┘
```

**Composants shadcn-vue** : `Input`, `Select`, `Button`, `Separator`, `Switch`
**Persistance** : `useStore()` composable → `tauri-plugin-store` → `preferences.json`

---

## Modal — Ollama non détecté

Affiché au démarrage si Ollama n'est pas disponible.

```
┌────────────────────────────────────────┐
│  ⚠️  Ollama non détecté               │
│                                        │
│  NeoGlot nécessite Ollama pour         │
│  effectuer les traductions localement. │
│                                        │
│  1. Installez Ollama :                 │
│     https://ollama.ai                  │
│  2. Démarrez Ollama :                  │
│     ollama serve                       │
│  3. Installez un modèle :              │
│     ollama pull llama3:8b              │
│                                        │
│  [Réessayer]   [Continuer sans IA]     │
└────────────────────────────────────────┘
```

**Composants shadcn-vue** : `AlertDialog`, `Button`

---

## Structure des fichiers Vue 3

```
src/
├── main.ts                      # createApp + Pinia + VueQueryPlugin + router + mount
├── App.vue                      # <RouterView /> + <Toaster /> (vue-sonner)
├── router/
│   └── index.ts                 # createRouter + createWebHashHistory (Tauri)
├── stores/                      # Pinia — état client partagé entre composants
│   ├── projectStore.ts          # currentProject (Project | null) — projet ouvert
│   ├── translationStore.ts      # isRunning, progress, percentage — job en cours
│   └── index.ts                 # barrel export
├── components/
│   ├── AppShell.vue             # Layout : AppSidebar + <main> + <RouterView>
│   ├── AppSidebar.vue           # Navigation latérale (shadcn-vue Sidebar)
│   ├── OllamaStatus.vue         # Badge connexion Ollama
│   └── ui/                      # shadcn-vue components (auto-générés)
├── views/                       # Pages Vue Router (route-level components, fins)
│   ├── HomeView.vue
│   ├── ProjectsView.vue
│   ├── TranslateView.vue
│   ├── GlossaryView.vue
│   └── SettingsView.vue
├── composables/                 # Logique réactive + TanStack Vue Query (state serveur)
│   ├── useOllama.ts             # check_ollama + list_ollama_models
│   ├── useProjects.ts           # list_projects, create_project, delete_project, useOpenProject
│   ├── useTranslate.ts          # useProjectFiles, useProjectStrings, useExtractProject (T05)
│   ├── useTranslation.ts        # start_translation via Channel → met à jour translationStore (T06)
│   ├── useGlossary.ts           # list_glossary, add/update/delete term
│   └── useStore.ts              # tauri-plugin-store prefs (settings persistés)
├── lib/
│   └── utils.ts                 # cn() helper Tailwind + utilitaires
└── types/
    ├── project.ts
    ├── string.ts
    └── glossary.ts
```

### Séparation des responsabilités d'état

| Besoin | Outil | Exemple |
|--------|-------|---------|
| Données serveur (invoke) | TanStack Vue Query | `useProjects()`, `useProjectProgress()` |
| État UI partagé cross-composants | Pinia | `useProjectStore`, `useTranslationStore` |
| Settings persistés sur disque | `useStore.ts` + LazyStore | URL Ollama, modèle, tokens/batch |
| État local à un composant | `shallowRef` / `ref` | Dialog ouvert/fermé, valeur input |
