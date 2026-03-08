<script setup lang="ts">
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { PlusIcon, FolderOpenIcon, LanguagesIcon } from 'lucide-vue-next'
import { useOllamaStatus } from '@/composables/useOllama'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import {
  AlertDialog,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'

const router = useRouter()
const { data: isOnline, isError, isPending } = useOllamaStatus()

// Modal bloquant si Ollama n'est pas détecté (et que la vérification est terminée)
const showOllamaModal = computed(
  () => !isPending.value && (isError.value || isOnline.value === false),
)
</script>

<template>
  <!-- Modal bloquant — non dismissible si Ollama absent -->
  <AlertDialog :open="showOllamaModal">
    <AlertDialogContent class="max-w-md">
      <AlertDialogHeader>
        <AlertDialogTitle class="flex items-center gap-2">
          <LanguagesIcon class="size-5 text-destructive" />
          Ollama non détecté
        </AlertDialogTitle>
        <AlertDialogDescription class="space-y-3 text-left">
          <p>NeoGlot nécessite <strong>Ollama</strong> pour fonctionner. Aucune connexion n'a pu être établie sur <code class="text-xs bg-muted px-1 rounded">localhost:11434</code>.</p>
          <p class="font-medium text-foreground">Pour installer Ollama :</p>
          <ol class="list-decimal list-inside text-sm space-y-1">
            <li>Rendez-vous sur <a href="https://ollama.com" target="_blank" class="text-primary underline">ollama.com</a></li>
            <li>Téléchargez et installez Ollama pour votre OS</li>
            <li>Lancez <code class="text-xs bg-muted px-1 rounded">ollama serve</code> dans un terminal</li>
            <li>Installez un modèle : <code class="text-xs bg-muted px-1 rounded">ollama pull gemma3</code></li>
          </ol>
          <p class="text-xs text-muted-foreground">La fenêtre se fermera automatiquement dès qu'Ollama sera détecté.</p>
        </AlertDialogDescription>
      </AlertDialogHeader>
    </AlertDialogContent>
  </AlertDialog>

  <!-- Contenu principal -->
  <div class="p-6 space-y-6">
    <div>
      <h1 class="text-2xl font-bold tracking-tight">Accueil</h1>
      <p class="text-muted-foreground">Traduisez vos jeux RPG open-source avec l'IA locale.</p>
    </div>

    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      <Card
        class="cursor-pointer hover:border-primary/50 transition-colors"
        @click="router.push('/projects')"
      >
        <CardHeader class="flex flex-row items-center gap-3 pb-2">
          <FolderOpenIcon class="size-5 text-muted-foreground" />
          <CardTitle class="text-base">Mes projets</CardTitle>
        </CardHeader>
        <CardContent>
          <CardDescription>Gérer et reprendre vos traductions en cours.</CardDescription>
        </CardContent>
      </Card>

      <Card
        class="cursor-pointer hover:border-primary/50 transition-colors"
        @click="router.push('/projects')"
      >
        <CardHeader class="flex flex-row items-center gap-3 pb-2">
          <PlusIcon class="size-5 text-muted-foreground" />
          <CardTitle class="text-base">Nouveau projet</CardTitle>
        </CardHeader>
        <CardContent>
          <CardDescription>Importer un nouveau jeu et démarrer une traduction.</CardDescription>
        </CardContent>
      </Card>
    </div>
  </div>
</template>
