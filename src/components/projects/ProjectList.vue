<script setup lang="ts">
import { FolderIcon } from 'lucide-vue-next'
import type { Project } from '@/types/project'
import { Skeleton } from '@/components/ui/skeleton'
import ProjectCard from './ProjectCard.vue'

defineProps<{
  projects: Project[]
  isLoading: boolean
}>()

const emit = defineEmits<{
  open: [project: Project]
  delete: [id: number]
}>()
</script>

<template>
  <div v-if="isLoading" class="space-y-3">
    <Skeleton class="h-36 w-full rounded-lg" />
    <Skeleton class="h-36 w-full rounded-lg" />
    <Skeleton class="h-36 w-full rounded-lg" />
  </div>

  <div
    v-else-if="projects.length === 0"
    class="flex flex-col items-center justify-center py-20 text-center gap-3"
  >
    <FolderIcon class="size-12 text-muted-foreground/40" />
    <p class="text-muted-foreground">Aucun projet pour l'instant.</p>
    <p class="text-sm text-muted-foreground/70">
      Cliquez sur « Nouveau projet » pour importer un jeu.
    </p>
  </div>

  <div v-else class="space-y-3">
    <ProjectCard
      v-for="project in projects"
      :key="project.id"
      :project="project"
      @open="emit('open', $event)"
      @delete="emit('delete', $event)"
    />
  </div>
</template>
