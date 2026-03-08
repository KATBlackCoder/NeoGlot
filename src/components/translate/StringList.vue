<script setup lang="ts">
import { computed, ref } from 'vue'
import { SearchIcon } from 'lucide-vue-next'
import type { StringEntry } from '@/composables/useTranslate'
import { STRING_STATUS_LABELS, STRING_STATUS_VARIANTS } from '@/types/project'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Skeleton } from '@/components/ui/skeleton'
import { Badge } from '@/components/ui/badge'
import { Input } from '@/components/ui/input'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'

const props = defineProps<{
  strings: StringEntry[]
  isLoading: boolean
}>()

const STATUS_FILTERS = ['all', 'pending', 'translated', 'reviewed'] as const
type StatusFilter = (typeof STATUS_FILTERS)[number]

const STATUS_FILTER_LABELS: Record<StatusFilter, string> = {
  all: 'Tous',
  pending: STRING_STATUS_LABELS.pending,
  translated: STRING_STATUS_LABELS.translated,
  reviewed: STRING_STATUS_LABELS.reviewed,
}

const search = ref('')
const statusFilter = ref<StatusFilter>('all')

const filtered = computed(() => {
  let list = props.strings
  if (statusFilter.value !== 'all') {
    list = list.filter((s) => s.status === statusFilter.value)
  }
  if (search.value.trim()) {
    const q = search.value.toLowerCase()
    list = list.filter(
      (s) =>
        s.source_text.toLowerCase().includes(q) ||
        (s.translation ?? '').toLowerCase().includes(q),
    )
  }
  return list
})
</script>

<template>
  <div class="flex flex-col h-full">
    <!-- Barre de filtres -->
    <div class="px-4 py-2 border-b border-border shrink-0 flex items-center gap-2 flex-wrap">
      <div class="relative flex-1 min-w-32 max-w-xs">
        <SearchIcon
          class="absolute left-2.5 top-1/2 -translate-y-1/2 size-3.5 text-muted-foreground"
        />
        <Input v-model="search" placeholder="Rechercher…" class="pl-8 h-8 text-sm" />
      </div>

      <ToggleGroup
        v-model="statusFilter"
        type="single"
        class="h-8"
      >
        <ToggleGroupItem
          v-for="s in STATUS_FILTERS"
          :key="s"
          :value="s"
          class="h-8 px-2.5 text-xs"
        >
          {{ STATUS_FILTER_LABELS[s] }}
        </ToggleGroupItem>
      </ToggleGroup>

      <span class="text-xs text-muted-foreground tabular-nums ml-auto">
        {{ filtered.length }} / {{ strings.length }}
      </span>
    </div>

    <!-- Liste des strings -->
    <ScrollArea class="flex-1">
      <div class="p-3 space-y-1.5">
        <!-- Squelettes -->
        <template v-if="isLoading">
          <Skeleton v-for="i in 8" :key="i" class="h-16 w-full rounded" />
        </template>

        <!-- État vide -->
        <div
          v-else-if="filtered.length === 0"
          class="flex items-center justify-center py-16 text-sm text-muted-foreground"
        >
          {{ strings.length === 0 ? 'Sélectionnez un fichier ou extrayez les textes.' : 'Aucun résultat.' }}
        </div>

        <!-- Entrées -->
        <div
          v-else
          v-for="s in filtered"
          :key="s.id"
          class="rounded-md border border-border bg-card p-3 space-y-1.5"
        >
          <div class="flex items-start justify-between gap-2">
            <p class="text-sm font-mono leading-snug whitespace-pre-wrap break-all flex-1">
              {{ s.source_text }}
            </p>
            <Badge
              :variant="STRING_STATUS_VARIANTS[s.status] ?? 'outline'"
              class="shrink-0 text-xs"
            >
              {{ STRING_STATUS_LABELS[s.status] ?? s.status }}
            </Badge>
          </div>
          <p
            v-if="s.translation"
            class="text-sm text-muted-foreground font-mono leading-snug whitespace-pre-wrap break-all border-t border-border/60 pt-1.5"
          >
            {{ s.translation }}
          </p>
        </div>
      </div>
    </ScrollArea>
  </div>
</template>
