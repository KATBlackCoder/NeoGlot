<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { PlusIcon, FolderOpenIcon, LanguagesIcon, BookOpenIcon, SettingsIcon } from 'lucide-vue-next'
import { useOllamaStatus } from '@/composables/useOllama'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import {
  AlertDialog,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'
import NewProjectDialog from '@/components/projects/NewProjectDialog.vue'

const router = useRouter()
const { data: isOnline, isError, isPending } = useOllamaStatus()

const showOllamaModal = computed(
  () => !isPending.value && (isError.value || isOnline.value === false),
)

const showNewProject = ref(false)

const QUICK_ACTIONS = [
  {
    icon: FolderOpenIcon,
    title: 'Mes projets',
    description: 'Gérer et reprendre vos traductions en cours.',
    action: () => router.push('/projects'),
  },
  {
    icon: PlusIcon,
    title: 'Nouveau projet',
    description: 'Importer un nouveau jeu et démarrer une traduction.',
    action: () => (showNewProject.value = true),
  },
  {
    icon: BookOpenIcon,
    title: 'Glossaire global',
    description: 'Gérer les termes partagés entre tous vos projets.',
    action: () => router.push('/glossary'),
  },
  {
    icon: SettingsIcon,
    title: 'Paramètres',
    description: 'Configurer Ollama et les préférences de traduction.',
    action: () => router.push('/settings'),
  },
]
</script>

<template>
  <!-- Modal bloquante si Ollama absent -->
  <AlertDialog :open="showOllamaModal">
    <AlertDialogContent class="max-w-md">
      <AlertDialogHeader>
        <AlertDialogTitle class="flex items-center gap-2">
          <LanguagesIcon class="size-5 text-destructive" />
          Ollama non détecté
        </AlertDialogTitle>
        <AlertDialogDescription class="space-y-3 text-left">
          <p>
            NeoGlot nécessite <strong>Ollama</strong> pour fonctionner. Aucune connexion n'a pu
            être établie sur
            <code class="text-xs bg-muted px-1 rounded">localhost:11434</code>.
          </p>
          <p class="font-medium text-foreground">Pour installer Ollama :</p>
          <ol class="list-decimal list-inside text-sm space-y-1">
            <li>
              Rendez-vous sur
              <a href="https://ollama.com" target="_blank" class="text-primary underline">
                ollama.com
              </a>
            </li>
            <li>Téléchargez et installez Ollama pour votre OS</li>
            <li>
              Lancez
              <code class="text-xs bg-muted px-1 rounded">ollama serve</code>
              dans un terminal
            </li>
            <li>
              Installez un modèle :
              <code class="text-xs bg-muted px-1 rounded">ollama pull gemma3</code>
            </li>
          </ol>
          <p class="text-xs text-muted-foreground">
            La fenêtre se fermera automatiquement dès qu'Ollama sera détecté.
          </p>
        </AlertDialogDescription>
      </AlertDialogHeader>
    </AlertDialogContent>
  </AlertDialog>

  <NewProjectDialog v-model:open="showNewProject" />

  <!-- Contenu principal -->
  <div class="p-6 space-y-8">
    <div>
      <h1 class="text-2xl font-bold tracking-tight">Accueil</h1>
      <p class="text-muted-foreground mt-1">
        Traduisez vos jeux RPG open-source avec l'IA locale.
      </p>
    </div>

    <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
      <Card
        v-for="item in QUICK_ACTIONS"
        :key="item.title"
        class="cursor-pointer group hover:border-primary/60 hover:shadow-sm transition-all duration-150"
        @click="item.action()"
      >
        <CardHeader class="flex flex-row items-center gap-3 pb-2">
          <div class="p-2 rounded-md bg-muted group-hover:bg-primary/10 transition-colors">
            <component :is="item.icon" class="size-4 text-muted-foreground group-hover:text-primary transition-colors" />
          </div>
          <CardTitle class="text-base">{{ item.title }}</CardTitle>
        </CardHeader>
        <CardContent>
          <CardDescription>{{ item.description }}</CardDescription>
        </CardContent>
      </Card>
    </div>
  </div>
</template>
