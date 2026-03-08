<script setup lang="ts">
import { computed } from 'vue'
import { FileTextIcon } from 'lucide-vue-next'
import type { FileEntry } from '@/composables/useTranslate'
import { calcPercent } from '@/lib/utils'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Skeleton } from '@/components/ui/skeleton'

const props = defineProps<{
  files: FileEntry[]
  isLoading: boolean
  selectedPath: string | null
}>()

const emit = defineEmits<{
  select: [path: string]
}>()

// Tri : maps d'abord, puis ordre alphabétique
const sortedFiles = computed(() =>
  [...props.files].sort((a, b) => {
    const aIsMap = a.relative_path.startsWith('maps')
    const bIsMap = b.relative_path.startsWith('maps')
    if (aIsMap && !bIsMap) return -1
    if (!aIsMap && bIsMap) return 1
    return a.relative_path.localeCompare(b.relative_path)
  }),
)
</script>

<template>
  <ScrollArea class="h-full">
    <div class="p-2 space-y-0.5">
      <p class="text-xs font-medium text-muted-foreground px-2 py-1.5 uppercase tracking-wide">
        Fichiers
      </p>

      <!-- Squelettes -->
      <template v-if="isLoading">
        <Skeleton v-for="i in 5" :key="i" class="h-9 w-full rounded" />
      </template>

      <!-- État vide -->
      <div
        v-else-if="files.length === 0"
        class="flex flex-col items-center justify-center py-8 gap-2 text-center"
      >
        <FileTextIcon class="size-8 text-muted-foreground/30" />
        <p class="text-xs text-muted-foreground">Aucun fichier extrait</p>
      </div>

      <!-- Liste -->
      <template v-else>
        <button
          v-for="file in sortedFiles"
          :key="file.relative_path"
          class="w-full flex items-center gap-2 px-2 py-1.5 rounded-sm text-left text-sm transition-colors hover:bg-accent hover:text-accent-foreground"
          :class="{ 'bg-accent text-accent-foreground font-medium': selectedPath === file.relative_path }"
          @click="emit('select', file.relative_path)"
        >
          <FileTextIcon class="size-3.5 shrink-0 text-muted-foreground" />
          <span class="flex-1 truncate text-xs">{{ file.relative_path }}</span>
          <span
            class="text-xs tabular-nums shrink-0"
            :class="
              calcPercent(file.strings_done, file.strings_total) === 100
                ? 'text-emerald-500 dark:text-emerald-400'
                : 'text-muted-foreground'
            "
          >
            {{ calcPercent(file.strings_done, file.strings_total) }}%
          </span>
        </button>
      </template>
    </div>
  </ScrollArea>
</template>
