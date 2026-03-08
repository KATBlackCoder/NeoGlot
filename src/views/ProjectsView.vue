<script setup lang="ts">
import { PlusIcon, FolderIcon } from 'lucide-vue-next'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Progress } from '@/components/ui/progress'
import { Badge } from '@/components/ui/badge'
import { Skeleton } from '@/components/ui/skeleton'

// Données factices — seront remplacées par invoke('list_projects') en T04
const isLoading = false
const projects: never[] = []
</script>

<template>
  <div class="p-6 space-y-6">
    <div class="flex items-center justify-between">
      <div>
        <h1 class="text-2xl font-bold tracking-tight">Projets</h1>
        <p class="text-muted-foreground">Gérez vos traductions de jeux RPG.</p>
      </div>
      <Button disabled>
        <PlusIcon data-icon="inline-start" />
        Nouveau projet
      </Button>
    </div>

    <!-- État de chargement -->
    <div v-if="isLoading" class="space-y-3">
      <Skeleton class="h-24 w-full rounded-lg" />
      <Skeleton class="h-24 w-full rounded-lg" />
    </div>

    <!-- Liste vide -->
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

    <!-- Liste des projets — sera alimentée en T04 -->
    <div v-else class="space-y-3">
      <Card v-for="project in projects" :key="(project as any).id">
        <CardHeader class="flex flex-row items-center justify-between pb-2">
          <div class="flex items-center gap-2">
            <CardTitle class="text-base">{{ (project as any).name }}</CardTitle>
            <Badge variant="outline">{{ (project as any).engine }}</Badge>
          </div>
          <Badge>{{ (project as any).status }}</Badge>
        </CardHeader>
        <CardContent>
          <CardDescription class="mb-2">{{ (project as any).game_path }}</CardDescription>
          <Progress :model-value="0" class="h-2" />
        </CardContent>
      </Card>
    </div>
  </div>
</template>
