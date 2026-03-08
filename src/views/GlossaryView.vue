<script setup lang="ts">
import { useRoute } from 'vue-router'
import { PlusIcon, BookOpenIcon } from 'lucide-vue-next'
import { Button } from '@/components/ui/button'
import {
  Table,
  TableBody,
  TableCell,
  TableEmpty,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { Badge } from '@/components/ui/badge'

const route = useRoute()
const projectId = route.params.id as string

// Données vides — seront chargées via invoke('list_glossary') en T08
const entries: never[] = []
</script>

<template>
  <div class="p-6 space-y-6">
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-2">
        <BookOpenIcon class="size-5 text-muted-foreground" />
        <div>
          <h1 class="text-2xl font-bold tracking-tight">Glossaire</h1>
          <p class="text-muted-foreground text-sm">Projet #{{ projectId }}</p>
        </div>
      </div>
      <div class="flex items-center gap-2">
        <Button variant="outline" size="sm" disabled>
          Importer speakers
        </Button>
        <Button size="sm" disabled>
          <PlusIcon data-icon="inline-start" />
          Ajouter un terme
        </Button>
      </div>
    </div>

    <Table>
      <TableHeader>
        <TableRow>
          <TableHead>Terme source</TableHead>
          <TableHead>Traduction</TableHead>
          <TableHead>Mode</TableHead>
          <TableHead>Note</TableHead>
          <TableHead class="w-20">Actions</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        <TableEmpty v-if="entries.length === 0">
          Aucun terme dans le glossaire. Ajoutez des termes pour guider la traduction.
        </TableEmpty>
        <TableRow v-for="entry in entries" :key="(entry as any).id">
          <TableCell class="font-medium">{{ (entry as any).term }}</TableCell>
          <TableCell>{{ (entry as any).translation }}</TableCell>
          <TableCell>
            <Badge variant="outline">{{ (entry as any).match_mode }}</Badge>
          </TableCell>
          <TableCell class="text-muted-foreground text-sm">{{ (entry as any).note }}</TableCell>
          <TableCell></TableCell>
        </TableRow>
      </TableBody>
    </Table>
  </div>
</template>
