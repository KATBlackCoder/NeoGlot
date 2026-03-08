<script setup lang="ts">
import { ref, computed, nextTick, watchEffect, onUnmounted } from 'vue'
import { useRoute } from 'vue-router'
import { PlayIcon, DownloadIcon, FolderSyncIcon, LoaderCircleIcon } from 'lucide-vue-next'
import { listen } from '@tauri-apps/api/event'
import { toast } from 'vue-sonner'
import { Button } from '@/components/ui/button'
import { Progress } from '@/components/ui/progress'
import { Badge } from '@/components/ui/badge'
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip'
import {
  AlertDialog,
  AlertDialogContent,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogDescription,
} from '@/components/ui/alert-dialog'
import {
  ResizablePanelGroup,
  ResizablePanel,
  ResizableHandle,
} from '@/components/ui/resizable'
import { useProjectStore } from '@/stores'
import { useProjectProgress } from '@/composables/useProjects'
import { useProjectFiles, useProjectStrings, useExtractProject } from '@/composables/useTranslate'
import { calcPercent } from '@/lib/utils'
import FileList from '@/components/translate/FileList.vue'
import StringList from '@/components/translate/StringList.vue'

const route = useRoute()
const projectId = Number(route.params.id)

const projectStore = useProjectStore()
const project = computed(() => projectStore.currentProject)

const { data: progress } = useProjectProgress(projectId)
const percentage = computed(() =>
  calcPercent(progress.value?.done ?? 0, progress.value?.total ?? 0),
)

const { data: files, isLoading: filesLoading } = useProjectFiles(projectId)

const selectedPath = ref<string | null>(null)

watchEffect(() => {
  if (files.value?.length && !selectedPath.value) {
    selectedPath.value = files.value[0].relative_path
  }
})

const { data: strings, isLoading: stringsLoading } = useProjectStrings(
  projectId,
  computed(() => selectedPath.value ?? undefined),
)

// ─── Extraction avec overlay bloquant ────────────────────────────────────────

interface ExtractionProgress {
  current: number
  total: number
  file: string
}

const extractionProgress = ref<ExtractionProgress | null>(null)
const isExtracting = ref(false)
const extractionPct = computed(() =>
  calcPercent(extractionProgress.value?.current ?? 0, extractionProgress.value?.total ?? 0),
)

const { mutate: extract } = useExtractProject()

let unlistenProgress: (() => void) | null = null
let dotsInterval: ReturnType<typeof setInterval> | null = null

// Barre de chargement indéterminée (0→95 en ~10s, puis attend le vrai progress)
const indeterminatePct = ref(0)
let indeterminateInterval: ReturnType<typeof setInterval> | null = null

// Points animés dans le titre
const dots = ref('.')

function startDots() {
  dots.value = '.'
  dotsInterval = setInterval(() => {
    dots.value = dots.value.length >= 3 ? '.' : dots.value + '.'
  }, 420)
}

function startIndeterminate() {
  indeterminatePct.value = 0
  indeterminateInterval = setInterval(() => {
    if (indeterminatePct.value < 90) {
      indeterminatePct.value += Math.random() * 3
    }
  }, 300)
}

function stopAnimations() {
  if (dotsInterval) { clearInterval(dotsInterval); dotsInterval = null }
  if (indeterminateInterval) { clearInterval(indeterminateInterval); indeterminateInterval = null }
  dots.value = '.'
  indeterminatePct.value = 0
}

async function handleExtract() {
  if (!project.value || isExtracting.value) return

  isExtracting.value = true
  extractionProgress.value = null
  startDots()
  startIndeterminate()

  // Force Vue à rendre le dialog AVANT de lancer l'extraction
  await nextTick()

  unlistenProgress = await listen<ExtractionProgress>('extraction-progress', (ev) => {
    extractionProgress.value = ev.payload
    // Arrêter la barre indéterminée dès qu'on a un vrai progress
    if (indeterminateInterval) { clearInterval(indeterminateInterval); indeterminateInterval = null }
  })

  extract(
    { projectId, gamePath: project.value.game_path, workPath: project.value.work_path },
    {
      onSuccess: (count) => {
        toast.success('Extraction terminée', {
          description: `${count} textes extraits avec succès.`,
        })
      },
      onError: (err) => {
        toast.error("Erreur lors de l'extraction", {
          description: String(err),
        })
      },
      onSettled: () => {
        unlistenProgress?.()
        unlistenProgress = null
        extractionProgress.value = null
        isExtracting.value = false
        stopAnimations()
      },
    },
  )
}

onUnmounted(() => {
  unlistenProgress?.()
  isExtracting.value = false
  stopAnimations()
})
</script>

<template>
  <div class="flex flex-col h-full">
    <!-- En-tête -->
    <div class="px-6 py-3 border-b border-border shrink-0 flex items-center justify-between gap-4">
      <div class="flex-1 min-w-0">
        <div class="flex items-center gap-2">
          <h1 class="text-base font-semibold truncate">
            {{ project?.name ?? `Projet #${projectId}` }}
          </h1>
          <Badge v-if="project?.engine" variant="outline" class="text-xs shrink-0">
            {{ project.engine }}
          </Badge>
        </div>
        <div class="flex items-center gap-2 mt-1">
          <Progress :model-value="percentage" class="h-1.5 max-w-xs" />
          <span class="text-xs text-muted-foreground tabular-nums">
            {{ progress?.done ?? 0 }} / {{ progress?.total ?? 0 }} ({{ percentage }}%)
          </span>
        </div>
      </div>

      <div class="flex items-center gap-2 shrink-0">
        <Button
          variant="outline"
          size="sm"
          :disabled="isExtracting || !project"
          @click="handleExtract"
        >
          <FolderSyncIcon class="size-4" />
          Extraire
        </Button>

        <Tooltip>
          <TooltipTrigger as-child>
            <span>
              <Button size="sm" disabled class="pointer-events-none">
                <PlayIcon class="size-4" />
                Traduire
              </Button>
            </span>
          </TooltipTrigger>
          <TooltipContent>Disponible après l'extraction des textes</TooltipContent>
        </Tooltip>

        <Tooltip>
          <TooltipTrigger as-child>
            <span>
              <Button variant="secondary" size="sm" disabled class="pointer-events-none">
                <DownloadIcon class="size-4" />
                Exporter
              </Button>
            </span>
          </TooltipTrigger>
          <TooltipContent>Disponible une fois la traduction terminée</TooltipContent>
        </Tooltip>
      </div>
    </div>

    <!-- Panneau fichiers | strings -->
    <ResizablePanelGroup direction="horizontal" class="flex-1 min-h-0">
      <ResizablePanel :default-size="22" :min-size="14" :max-size="38">
        <FileList
          :files="files ?? []"
          :is-loading="filesLoading"
          :selected-path="selectedPath"
          @select="selectedPath = $event"
        />
      </ResizablePanel>

      <ResizableHandle />

      <ResizablePanel :default-size="78">
        <StringList
          :strings="strings ?? []"
          :is-loading="stringsLoading"
        />
      </ResizablePanel>
    </ResizablePanelGroup>

    <!-- Overlay bloquant pendant l'extraction -->
    <AlertDialog :open="isExtracting">
      <AlertDialogContent
        class="max-w-sm"
        @pointer-down-outside.prevent
        @escape-key-down.prevent
        @interact-outside.prevent
      >
        <AlertDialogHeader>
          <AlertDialogTitle class="flex items-center gap-2">
            <LoaderCircleIcon class="size-5 animate-spin text-primary" />
            Extraction en cours<span class="inline-block w-5 text-primary">{{ dots }}</span>
          </AlertDialogTitle>

          <AlertDialogDescription as="div" class="space-y-3 pt-3">
            <!-- Phase : progress connu -->
            <template v-if="extractionProgress">
              <div class="flex items-center justify-between text-sm">
                <span class="text-muted-foreground truncate max-w-52" :title="extractionProgress.file">
                  {{ extractionProgress.file }}
                </span>
                <span class="tabular-nums font-medium shrink-0 text-foreground">
                  {{ extractionProgress.current }} / {{ extractionProgress.total }}
                </span>
              </div>
              <Progress :model-value="extractionPct" class="h-2" />
              <p class="text-xs text-muted-foreground text-center">
                {{ extractionPct }}% — Ne fermez pas l'application
              </p>
            </template>

            <!-- Phase : préparation (barre indéterminée) -->
            <template v-else>
              <div class="flex items-center gap-2 text-sm text-muted-foreground animate-pulse">
                <span>Préparation des fichiers</span>
                <span class="inline-block w-4">{{ dots }}</span>
              </div>
              <div class="relative h-2 w-full overflow-hidden rounded-full bg-secondary">
                <div class="extraction-indeterminate h-full w-1/3 rounded-full bg-primary/70" />
              </div>
            </template>
          </AlertDialogDescription>
        </AlertDialogHeader>
      </AlertDialogContent>
    </AlertDialog>
  </div>
</template>
