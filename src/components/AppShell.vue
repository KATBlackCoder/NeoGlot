<script setup lang="ts">
import { SidebarProvider, SidebarTrigger } from '@/components/ui/sidebar'
import AppSidebar from './AppSidebar.vue'
import OllamaStatus from './OllamaStatus.vue'
</script>

<template>
  <SidebarProvider>
    <AppSidebar />
    <main class="flex flex-col flex-1 min-h-svh overflow-hidden">
      <header class="flex items-center justify-between px-4 py-2 border-b border-border shrink-0">
        <SidebarTrigger />
        <OllamaStatus />
      </header>
      <div class="flex-1 overflow-auto">
        <RouterView />
      </div>
    </main>
  </SidebarProvider>
</template>
