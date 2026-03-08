<script setup lang="ts">
import { useRoute } from 'vue-router'
import { PlayIcon, PauseIcon, DownloadIcon } from 'lucide-vue-next'
import { Button } from '@/components/ui/button'
import { Progress } from '@/components/ui/progress'
import {
  ResizablePanelGroup,
  ResizablePanel,
  ResizableHandle,
} from '@/components/ui/resizable'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Badge } from '@/components/ui/badge'

const route = useRoute()
const projectId = route.params.id as string
</script>

<template>
  <div class="flex flex-col h-full">
    <!-- En-tête projet -->
    <div class="px-6 py-4 border-b border-border shrink-0 flex items-center justify-between gap-4">
      <div class="flex-1 min-w-0">
        <h1 class="text-lg font-semibold truncate">Traduction — Projet #{{ projectId }}</h1>
        <Progress :model-value="0" class="h-1.5 mt-1 max-w-xs" />
      </div>
      <div class="flex items-center gap-2 shrink-0">
        <Button variant="outline" size="sm" disabled>
          <PauseIcon data-icon="inline-start" />
          Pause
        </Button>
        <Button size="sm" disabled>
          <PlayIcon data-icon="inline-start" />
          Traduire
        </Button>
        <Button variant="secondary" size="sm" disabled>
          <DownloadIcon data-icon="inline-start" />
          Exporter
        </Button>
      </div>
    </div>

    <!-- Panneau redimensionnable fichiers | strings -->
    <ResizablePanelGroup direction="horizontal" class="flex-1 min-h-0">
      <ResizablePanel :default-size="25" :min-size="15" :max-size="40">
        <ScrollArea class="h-full">
          <div class="p-3 space-y-1">
            <p class="text-xs font-medium text-muted-foreground px-2 py-1">Fichiers</p>
            <div class="rounded-md bg-muted/50 px-3 py-2 text-sm text-muted-foreground italic">
              Disponible en T05
            </div>
          </div>
        </ScrollArea>
      </ResizablePanel>

      <ResizableHandle />

      <ResizablePanel :default-size="75">
        <ScrollArea class="h-full">
          <div class="p-4 space-y-2">
            <p class="text-xs font-medium text-muted-foreground">Chaînes de texte</p>
            <div class="flex flex-col items-center justify-center py-16 text-center gap-2">
              <Badge variant="outline">Extraction requise</Badge>
              <p class="text-sm text-muted-foreground">
                Extrayez d'abord les textes du jeu pour les afficher ici.
              </p>
            </div>
          </div>
        </ScrollArea>
      </ResizablePanel>
    </ResizablePanelGroup>
  </div>
</template>
