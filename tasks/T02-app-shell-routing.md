# T02 — App Shell, Routing et Layout

**Statut** : DONE
**Dépendances** : T01 (shadcn-vue + plugins installés)

---

## Objectif

Mettre en place la structure Vue 3 de base : App Shell avec sidebar, Vue Router, Pinia, VueQueryPlugin, thème sombre.
Tout en SFCs `.vue` avec `<script setup lang="ts">`.

---

## Étapes

### 1. Configurer le thème sombre dans src/style.css

```css
@import "tailwindcss";
@import "tw-animate-css";

@custom-variant dark (&:is(.dark *));

:root {
  /* variables shadcn-vue zinc */
}

.dark { ... }
```

`index.html` : ajouter `class="dark"` sur `<html>` pour le thème sombre par défaut.

### 2. Créer la structure de dossiers

```
src/
├── components/
│   ├── ui/                  (généré par shadcn-vue)
│   ├── AppShell.vue
│   ├── AppSidebar.vue
│   └── OllamaStatus.vue
├── views/                   (pages Vue Router — fines, composition surface)
│   ├── HomeView.vue
│   ├── ProjectsView.vue
│   ├── TranslateView.vue
│   ├── GlossaryView.vue
│   └── SettingsView.vue
├── composables/             (logique réactive + Vue Query)
│   ├── useOllama.ts
│   └── useStore.ts
├── stores/                  (Pinia — état client partagé)
│   ├── projectStore.ts      (projet ouvert : currentProject)
│   ├── translationStore.ts  (job en cours : isRunning, progress)
│   └── index.ts             (barrel export)
├── router/
│   └── index.ts
├── lib/
│   └── utils.ts             (cn() helper Tailwind)
└── types/
    ├── project.ts
    └── glossary.ts
```

### 3. Créer src/types/project.ts

```typescript
export interface Project {
  id: number;
  name: string;
  game_path: string;
  work_path: string;
  engine: string;
  source_lang: string;
  target_lang: string;
  project_context: string;
  status: string;
  created_at: string;
}
```

### 4. Créer src/types/glossary.ts

```typescript
export interface GlossaryEntry {
  id: number;
  project_id: number | null; // null = glossaire global
  term: string;
  translation: string;
  note: string;
  match_mode: 'exact' | 'contains' | 'regex';
}
```

Note : pas de `src/lib/db.ts`. Toutes les données viennent de `invoke()`.
Le schéma SQLite est initialisé côté Rust au démarrage (`db::init_schema()` dans T03).

### 5. Créer src/router/index.ts

```typescript
import { createRouter, createWebHashHistory } from 'vue-router';

// createWebHashHistory obligatoire dans Tauri (pas de serveur HTTP)
const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    {
      path: '/',
      component: () => import('@/components/AppShell.vue'),
      children: [
        { path: '', component: () => import('@/views/HomeView.vue') },
        { path: 'projects', component: () => import('@/views/ProjectsView.vue') },
        { path: 'projects/:id/translate', component: () => import('@/views/TranslateView.vue') },
        { path: 'projects/:id/glossary', component: () => import('@/views/GlossaryView.vue') },
        { path: 'settings', component: () => import('@/views/SettingsView.vue') },
      ],
    },
  ],
});

export default router;
```

### 6. Créer src/composables/useOllama.ts

Toutes les vérifications Ollama passent par `invoke()` vers Rust (`ollama-rs`).

```typescript
import { invoke } from '@tauri-apps/api/core';
import { useQuery } from '@tanstack/vue-query';

export function useOllamaStatus() {
  return useQuery({
    queryKey: ['ollama-status'],
    queryFn: () => invoke<boolean>('check_ollama'),
    retry: false,
    refetchInterval: 30_000,
  });
}

export function useOllamaModels() {
  return useQuery({
    queryKey: ['ollama-models'],
    queryFn: () => invoke<string[]>('list_ollama_models'),
    retry: false,
  });
}
```

### 7. Créer src/components/OllamaStatus.vue

```vue
<script setup lang="ts">
import { useOllamaStatus } from '@/composables/useOllama';
import { Badge } from '@/components/ui/badge';

const { data: isOnline, isError, isPending } = useOllamaStatus();
</script>

<template>
  <Badge v-if="isPending" variant="outline">Ollama...</Badge>
  <Badge v-else-if="isError || !isOnline" variant="destructive">Ollama hors ligne</Badge>
  <Badge v-else class="bg-green-600">Ollama ✓</Badge>
</template>
```

### 8. Créer src/components/AppSidebar.vue

Navigation latérale avec les liens principaux.
Utiliser le composant `Sidebar` de shadcn-vue.

```vue
<script setup lang="ts">
import { useRouter, useRoute } from 'vue-router';
// Sidebar, SidebarContent, SidebarMenu, etc. depuis shadcn-vue
</script>

<template>
  <Sidebar>
    <SidebarContent>
      <SidebarMenu>
        <SidebarMenuItem v-for="item in navItems" :key="item.path">
          <SidebarMenuButton :is-active="route.path === item.path" @click="router.push(item.path)">
            {{ item.label }}
          </SidebarMenuButton>
        </SidebarMenuItem>
      </SidebarMenu>
    </SidebarContent>
  </Sidebar>
</template>
```

### 9. Créer src/components/AppShell.vue

Layout principal : AppSidebar + zone de contenu + `<RouterView>`.

```vue
<script setup lang="ts">
import AppSidebar from './AppSidebar.vue';
import OllamaStatus from './OllamaStatus.vue';
import { SidebarProvider } from '@/components/ui/sidebar';
</script>

<template>
  <SidebarProvider>
    <AppSidebar />
    <main class="flex-1 overflow-auto">
      <header class="flex items-center justify-end p-2 border-b">
        <OllamaStatus />
      </header>
      <RouterView />
    </main>
  </SidebarProvider>
</template>
```

### 10. Mettre à jour src/main.ts

```typescript
import { createApp } from 'vue';
import { createPinia } from 'pinia';
import { VueQueryPlugin } from '@tanstack/vue-query';
import App from './App.vue';
import router from './router';
import './style.css';

const app = createApp(App);
app.use(createPinia()); // Pinia avant router (stores accessibles dans les navigation guards)
app.use(router);
app.use(VueQueryPlugin);
app.mount('#app');
```

### 11. Mettre à jour src/App.vue

```vue
<script setup lang="ts">
// Racine minimaliste — juste RouterView
</script>

<template>
  <RouterView />
</template>
```

### 12. Modal Ollama non détecté

Dans `HomeView.vue`, si `useOllamaStatus()` retourne `false` ou une erreur, afficher un `AlertDialog` non dismissible avec les instructions d'installation.

```vue
<script setup lang="ts">
import { useOllamaStatus } from '@/composables/useOllama';
import { AlertDialog, AlertDialogContent, ... } from '@/components/ui/alert-dialog';

const { data: isOnline, isError } = useOllamaStatus();
const showOllamaModal = computed(() => isError.value || isOnline.value === false);
</script>

<template>
  <AlertDialog :open="showOllamaModal">
    <AlertDialogContent>
      <!-- instructions Ollama -->
    </AlertDialogContent>
  </AlertDialog>
  <!-- reste de la vue -->
</template>
```

---

## Fichiers créés/modifiés

- `src/main.ts` — Pinia + VueQueryPlugin + router + mount
- `src/App.vue` — `<RouterView />` racine
- `src/router/index.ts` — routes avec `createWebHashHistory`
- `src/types/project.ts` / `glossary.ts` — types TypeScript
- `src/composables/useOllama.ts` — status Ollama via `invoke('check_ollama')`
- `src/composables/useStore.ts` — settings persistés via `LazyStore`
- `src/stores/projectStore.ts` — store Pinia projet ouvert
- `src/stores/translationStore.ts` — store Pinia job de traduction
- `src/stores/index.ts` — barrel export
- `src/components/OllamaStatus.vue` — badge status
- `src/components/AppSidebar.vue` — navigation
- `src/components/AppShell.vue` — layout
- `src/views/HomeView.vue` — accueil + modal Ollama bloquant
- `src/views/ProjectsView.vue` — scaffold liste projets
- `src/views/TranslateView.vue` — scaffold ResizablePanelGroup
- `src/views/GlossaryView.vue` — scaffold Table
- `src/views/SettingsView.vue` — config Ollama + tokens/batch

---

## Validation

- Navigation entre toutes les vues sans erreur
- Badge Ollama vert si Ollama est démarré, rouge sinon
- Thème sombre appliqué partout
- `createWebHashHistory` — URLs `/#/projects` (pas de 404 Tauri)
