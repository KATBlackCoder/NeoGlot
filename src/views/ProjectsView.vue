<script setup lang="ts">
import { ref } from 'vue'
import { PlusIcon } from 'lucide-vue-next'
import { Button } from '@/components/ui/button'
import { useProjects, useDeleteProject, useOpenProject } from '@/composables/useProjects'
import ProjectList from '@/components/projects/ProjectList.vue'
import NewProjectDialog from '@/components/projects/NewProjectDialog.vue'

const { data: projects, isLoading } = useProjects()
const { mutate: deleteProject } = useDeleteProject()
const openProject = useOpenProject()

const isDialogOpen = ref(false)
</script>

<template>
  <div class="p-6 space-y-6">
    <div class="flex items-center justify-between">
      <div>
        <h1 class="text-2xl font-bold tracking-tight">Projets</h1>
        <p class="text-muted-foreground">Gérez vos traductions de jeux RPG.</p>
      </div>
      <Button @click="isDialogOpen = true" class="gap-1.5">
        <PlusIcon class="size-4" />
        Nouveau projet
      </Button>
    </div>

    <ProjectList
      :projects="projects ?? []"
      :is-loading="isLoading"
      @open="openProject"
      @delete="deleteProject"
    />

    <NewProjectDialog v-model:open="isDialogOpen" />
  </div>
</template>
