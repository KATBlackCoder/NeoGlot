<script setup lang="ts">
import { computed } from 'vue'
import { FolderOpenIcon, Trash2Icon } from 'lucide-vue-next'
import type { Project } from '@/types/project'
import { ENGINE_LABELS, PROJECT_STATUS_LABELS, PROJECT_STATUS_VARIANTS } from '@/types/project'
import { useProjectProgress } from '@/composables/useProjects'
import { calcPercent } from '@/lib/utils'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Progress } from '@/components/ui/progress'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from '@/components/ui/alert-dialog'

const props = defineProps<{ project: Project }>()

const emit = defineEmits<{
  open: [project: Project]
  delete: [id: number]
}>()

const { data: progress } = useProjectProgress(props.project.id)

const percentage = computed(() =>
  calcPercent(progress.value?.done ?? 0, progress.value?.total ?? 0),
)
</script>

<template>
  <Card class="transition-shadow hover:shadow-md">
    <CardHeader class="flex flex-row items-start justify-between pb-2 gap-4">
      <div class="flex flex-col gap-1 min-w-0">
        <div class="flex items-center gap-2 flex-wrap">
          <CardTitle class="text-base truncate">{{ project.name }}</CardTitle>
          <Badge variant="outline" class="shrink-0 text-xs">
            {{ ENGINE_LABELS[project.engine] ?? project.engine }}
          </Badge>
        </div>
        <p class="text-xs text-muted-foreground truncate">{{ project.game_path }}</p>
      </div>
      <Badge :variant="PROJECT_STATUS_VARIANTS[project.status] ?? 'secondary'" class="shrink-0">
        {{ PROJECT_STATUS_LABELS[project.status] ?? project.status }}
      </Badge>
    </CardHeader>

    <CardContent class="space-y-3">
      <div class="space-y-1">
        <div class="flex justify-between text-xs text-muted-foreground">
          <span>{{ progress?.done ?? 0 }} / {{ progress?.total ?? 0 }} chaînes</span>
          <span>{{ percentage }}%</span>
        </div>
        <Progress :model-value="percentage" class="h-1.5" />
      </div>

      <div class="flex gap-2">
        <Button size="sm" class="flex-1 gap-1.5" @click="emit('open', project)">
          <FolderOpenIcon class="size-4" />
          Ouvrir
        </Button>

        <AlertDialog>
          <AlertDialogTrigger as-child>
            <Button
              size="sm"
              variant="outline"
              class="text-destructive hover:bg-destructive hover:text-destructive-foreground"
            >
              <Trash2Icon class="size-4" />
            </Button>
          </AlertDialogTrigger>
          <AlertDialogContent>
            <AlertDialogHeader>
              <AlertDialogTitle>Supprimer le projet ?</AlertDialogTitle>
              <AlertDialogDescription>
                Cette action supprime définitivement <strong>{{ project.name }}</strong> et toutes
                ses traductions. Elle est irréversible.
              </AlertDialogDescription>
            </AlertDialogHeader>
            <AlertDialogFooter>
              <AlertDialogCancel>Annuler</AlertDialogCancel>
              <AlertDialogAction
                class="bg-destructive text-destructive-foreground hover:bg-destructive/90"
                @click="emit('delete', project.id)"
              >
                Supprimer
              </AlertDialogAction>
            </AlertDialogFooter>
          </AlertDialogContent>
        </AlertDialog>
      </div>
    </CardContent>
  </Card>
</template>
