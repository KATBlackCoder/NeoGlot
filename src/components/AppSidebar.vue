<script setup lang="ts">
import { useRoute, useRouter } from 'vue-router'
import {
  HomeIcon,
  FolderOpenIcon,
  SettingsIcon,
  LanguagesIcon,
} from 'lucide-vue-next'
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from '@/components/ui/sidebar'

const route = useRoute()
const router = useRouter()

const navItems = [
  { path: '/', label: 'Accueil', icon: HomeIcon, exact: true },
  { path: '/projects', label: 'Projets', icon: FolderOpenIcon, exact: false },
  { path: '/settings', label: 'Paramètres', icon: SettingsIcon, exact: false },
]

// Détermine si un item de nav est actif selon le chemin courant
function isActive(item: { path: string; exact: boolean }) {
  if (item.exact) return route.path === item.path
  return route.path.startsWith(item.path)
}
</script>

<template>
  <Sidebar>
    <SidebarHeader class="p-4">
      <div class="flex items-center gap-2">
        <LanguagesIcon class="size-6 text-sidebar-primary" />
        <span class="text-base font-semibold text-sidebar-foreground">NeoGlot</span>
      </div>
    </SidebarHeader>

    <SidebarContent>
      <SidebarMenu>
        <SidebarMenuItem v-for="item in navItems" :key="item.path">
          <SidebarMenuButton
            :is-active="isActive(item)"
            @click="router.push(item.path)"
            class="cursor-pointer"
          >
            <component :is="item.icon" />
            <span>{{ item.label }}</span>
          </SidebarMenuButton>
        </SidebarMenuItem>
      </SidebarMenu>
    </SidebarContent>

    <SidebarFooter class="p-3">
      <p class="text-xs text-sidebar-foreground/50 text-center">v0.1.0</p>
    </SidebarFooter>
  </Sidebar>
</template>
