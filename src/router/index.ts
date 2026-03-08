import { createRouter, createWebHashHistory } from 'vue-router'

// createWebHashHistory obligatoire dans Tauri — pas de serveur HTTP
const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    {
      path: '/',
      component: () => import('@/components/AppShell.vue'),
      children: [
        { path: '', component: () => import('@/views/HomeView.vue') },
        { path: 'projects', component: () => import('@/views/ProjectsView.vue') },
        { path: 'projects/:id/translate', component: () => import('@/views/TranslateView.vue') },
        { path: 'projects/:id/glossary', component: () => import('@/views/GlossaryView.vue') },
        { path: 'settings', component: () => import('@/views/SettingsView.vue') },
      ],
    },
  ],
})

export default router
