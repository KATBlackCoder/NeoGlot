<script setup lang="ts">
import { useOllamaStatus } from '@/composables/useOllama'
import { Badge } from '@/components/ui/badge'

const { data: isOnline, isError, isPending } = useOllamaStatus()
</script>

<template>
  <Badge v-if="isPending" variant="secondary" class="text-xs">
    Ollama…
  </Badge>
  <Badge v-else-if="isError || !isOnline" variant="destructive" class="text-xs">
    Ollama hors ligne
  </Badge>
  <Badge
    v-else
    variant="outline"
    class="text-xs border-emerald-500/50 text-emerald-600 dark:text-emerald-400"
  >
    Ollama ✓
  </Badge>
</template>
