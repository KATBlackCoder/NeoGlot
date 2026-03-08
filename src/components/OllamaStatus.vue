<script setup lang="ts">
import { useOllamaStatus } from '@/composables/useOllama'
import { Badge } from '@/components/ui/badge'

const { data: isOnline, isError, isPending } = useOllamaStatus()
</script>

<template>
  <Badge v-if="isPending" variant="outline" class="text-xs">
    Ollama...
  </Badge>
  <Badge v-else-if="isError || !isOnline" variant="destructive" class="text-xs">
    Ollama hors ligne
  </Badge>
  <Badge v-else class="bg-green-600 text-white hover:bg-green-700 text-xs">
    Ollama ✓
  </Badge>
</template>
