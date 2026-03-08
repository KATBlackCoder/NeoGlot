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
import type { GlossaryEntry } from '@/types/glossary'

const route = useRoute()
const projectId = route.params.id as string

// Données vides — seront chargées via invoke('list_glossary') en T08
const entries: GlossaryEntry[] = []
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
        <Button size="sm" disabled class="gap-1.5">
          <PlusIcon class="size-4" />
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
        <TableRow v-for="entry in entries" :key="entry.id">
          <TableCell class="font-medium">{{ entry.term }}</TableCell>
          <TableCell>{{ entry.translation }}</TableCell>
          <TableCell>
            <Badge variant="outline">{{ entry.match_mode }}</Badge>
          </TableCell>
          <TableCell class="text-muted-foreground text-sm">{{ entry.note }}</TableCell>
          <TableCell></TableCell>
        </TableRow>
      </TableBody>
    </Table>
  </div>
</template>
