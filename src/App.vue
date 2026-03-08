<script setup lang="ts">
import { Toaster } from '@/components/ui/sonner'
</script>

<template>
  <RouterView />
  <Toaster theme="dark" rich-colors position="bottom-right" />
</template>
