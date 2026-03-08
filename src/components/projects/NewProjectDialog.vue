<script setup lang="ts">
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open as openDialog } from '@tauri-apps/plugin-dialog'
import { useQueryClient } from '@tanstack/vue-query'
import { useRouter } from 'vue-router'
import { FolderSearchIcon, LoaderCircleIcon, CheckCircleIcon, XIcon, RotateCcwIcon } from 'lucide-vue-next'
import { useProjectStore } from '@/stores'
import { ENGINE_LABELS } from '@/types/project'
import type { Project } from '@/types/project'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
  DialogClose,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import { Badge } from '@/components/ui/badge'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'

defineProps<{ open: boolean }>()
const emit = defineEmits<{ 'update:open': [value: boolean] }>()

const qc = useQueryClient()
const router = useRouter()
const projectStore = useProjectStore()

// ─── État du formulaire ───────────────────────────────────────────────────────

const name = ref('')
const gamePath = ref('')
const detectedEngine = ref<string | null>(null)
const sourceLang = ref('ja')
const targetLang = ref('fr')
const projectContext = ref('')
const isDetecting = ref(false)
const isCreating = ref(false)
const detectError = ref<string | null>(null)
const createError = ref<string | null>(null)

const LANG_OPTIONS = [
  { value: 'ja', label: 'Japonais' },
  { value: 'en', label: 'Anglais' },
  { value: 'fr', label: 'Français' },
  { value: 'zh', label: 'Chinois' },
  { value: 'ko', label: 'Coréen' },
  { value: 'de', label: 'Allemand' },
  { value: 'es', label: 'Espagnol' },
]

const canSubmit = computed(
  () =>
    name.value.trim() !== '' &&
    gamePath.value !== '' &&
    detectedEngine.value !== null &&
    !isCreating.value,
)

// ─── Actions ──────────────────────────────────────────────────────────────────

async function pickGameFolder() {
  const dir = await openDialog({ directory: true, title: 'Dossier racine du jeu' })
  if (!dir) return

  gamePath.value = dir as string

  // Toujours écraser le nom avec le nom du dossier (max 70 caractères)
  const folderName = (dir as string).replace(/\\/g, '/').split('/').pop() ?? ''
  name.value = folderName.slice(0, 70)

  detectedEngine.value = null
  detectError.value = null
  isDetecting.value = true

  try {
    detectedEngine.value = await invoke<string>('detect_engine', { gamePath: gamePath.value })
  } catch (err) {
    detectError.value = String(err)
  } finally {
    isDetecting.value = false
  }
}

async function handleCreate() {
  if (!canSubmit.value) return
  isCreating.value = true
  createError.value = null

  try {
    // Dossier de travail à l'intérieur du jeu (séparateur adapté à l'OS)
    const sep = gamePath.value.includes('\\') ? '\\' : '/'
    const workPath = `${gamePath.value}${sep}.neoglot`
    const project = await invoke<Project>('create_project', {
      name: name.value.trim(),
      gamePath: gamePath.value,
      workPath,
      engine: detectedEngine.value,
      sourceLang: sourceLang.value,
      targetLang: targetLang.value,
      projectContext: projectContext.value,
    })

    await qc.invalidateQueries({ queryKey: ['projects'] })
    emit('update:open', false)
    projectStore.setProject(project)
    router.push(`/projects/${project.id}/translate`)
  } catch (err) {
    createError.value = String(err)
  } finally {
    isCreating.value = false
  }
}

function resetForm() {
  name.value = ''
  gamePath.value = ''
  detectedEngine.value = null
  sourceLang.value = 'ja'
  targetLang.value = 'fr'
  projectContext.value = ''
  detectError.value = null
  createError.value = null
}
</script>

<template>
  <Dialog
    :open="open"
    @update:open="
      (v) => {
        emit('update:open', v)
        if (!v) resetForm()
      }
    "
  >
    <DialogContent class="sm:max-w-lg">
      <DialogHeader>
        <DialogTitle>Nouveau projet</DialogTitle>
      </DialogHeader>

      <div class="space-y-4 py-2">
        <!-- Nom du projet -->
        <div class="space-y-1.5">
          <label class="text-sm font-medium" for="project-name">Nom du projet</label>
          <div class="relative">
            <Input id="project-name" v-model="name" placeholder="Mon jeu RPG" class="pr-8" maxlength="70" />
            <button
              v-if="name"
              type="button"
              class="absolute right-2 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground transition-colors"
              @click="name = ''"
            >
              <XIcon class="size-3.5" />
            </button>
          </div>
        </div>

        <!-- Dossier du jeu -->
        <div class="space-y-1.5">
          <label class="text-sm font-medium" for="game-path">Dossier racine du jeu</label>
          <div class="flex gap-2">
            <Input
              id="game-path"
              :model-value="gamePath"
              readonly
              placeholder="Cliquer pour sélectionner…"
              class="flex-1 cursor-pointer"
              @click="pickGameFolder"
            />
            <Button
              v-if="gamePath"
              variant="outline"
              size="icon"
              title="Réinitialiser le dossier"
              @click="gamePath = ''; detectedEngine = null; detectError = null; name = ''"
            >
              <RotateCcwIcon class="size-4" />
            </Button>
            <Button variant="outline" size="icon" :disabled="isDetecting" @click="pickGameFolder">
              <LoaderCircleIcon v-if="isDetecting" class="size-4 animate-spin" />
              <FolderSearchIcon v-else class="size-4" />
            </Button>
          </div>

          <!-- Résultat de détection -->
          <div
            v-if="detectedEngine"
            class="flex items-center gap-1.5 text-xs text-muted-foreground"
          >
            <CheckCircleIcon class="size-3.5 text-green-500" />
            <span>Moteur détecté :</span>
            <Badge variant="outline" class="text-xs">
              {{ ENGINE_LABELS[detectedEngine] ?? detectedEngine }}
            </Badge>
          </div>
          <p v-if="detectError" class="text-xs text-destructive">{{ detectError }}</p>
        </div>

        <!-- Langues -->
        <div class="grid grid-cols-2 gap-3">
          <div class="space-y-1.5">
            <label class="text-sm font-medium" for="source-lang">Langue source</label>
            <Select v-model="sourceLang">
              <SelectTrigger id="source-lang">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem v-for="l in LANG_OPTIONS" :key="l.value" :value="l.value">
                  {{ l.label }}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div class="space-y-1.5">
            <label class="text-sm font-medium" for="target-lang">Langue cible</label>
            <Select v-model="targetLang">
              <SelectTrigger id="target-lang">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem v-for="l in LANG_OPTIONS" :key="l.value" :value="l.value">
                  {{ l.label }}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>

        <!-- Contexte optionnel -->
        <div class="space-y-1.5">
          <label class="text-sm font-medium" for="project-context">
            Contexte du projet
            <span class="text-muted-foreground font-normal">(optionnel)</span>
          </label>
          <Textarea
            id="project-context"
            v-model="projectContext"
            placeholder="Style de traduction attendu, noms propres à conserver…"
            :rows="3"
          />
        </div>

        <p v-if="createError" class="text-xs text-destructive">{{ createError }}</p>
      </div>

      <DialogFooter>
        <DialogClose as-child>
          <Button variant="outline" @click="resetForm">Annuler</Button>
        </DialogClose>
        <Button :disabled="!canSubmit" @click="handleCreate">
          <LoaderCircleIcon v-if="isCreating" class="size-4 mr-1 animate-spin" />
          Créer le projet
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
