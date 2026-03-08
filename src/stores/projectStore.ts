import { defineStore } from 'pinia'
import { shallowRef, computed } from 'vue'
import type { Project } from '@/types/project'

// Store du projet actuellement ouvert
// Évite le prop-drilling de l'id et des données projet entre TranslateView / GlossaryView / AppSidebar
export const useProjectStore = defineStore('project', () => {
  const currentProject = shallowRef<Project | null>(null)

  const hasProject = computed(() => currentProject.value !== null)

  function setProject(project: Project) {
    currentProject.value = project
  }

  function clearProject() {
    currentProject.value = null
  }

  return {
    currentProject,
    hasProject,
    setProject,
    clearProject,
  }
})
