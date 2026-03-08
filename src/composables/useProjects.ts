import { invoke } from '@tauri-apps/api/core'
import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query'
import { useRouter } from 'vue-router'
import type { Project } from '@/types/project'
import { useProjectStore } from '@/stores'

// ─── Labels lisibles par moteur ──────────────────────────────────────────────

export const ENGINE_LABELS: Record<string, string> = {
  rpgmz: 'RPG Maker MZ',
  rpgmv: 'RPG Maker MV',
  rpgmxp: 'RPG Maker XP',
  rpgmvx: 'RPG Maker VX',
  rpgmvxa: 'RPG Maker VXAce',
  wolf: 'Wolf RPG',
  bakin: 'RPG Bakin',
  unknown: 'Inconnu',
}

// ─── Composables ─────────────────────────────────────────────────────────────

export function useProjects() {
  return useQuery({
    queryKey: ['projects'],
    queryFn: () => invoke<Project[]>('list_projects'),
  })
}

export function useDeleteProject() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (id: number) => invoke('delete_project', { projectId: id }),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['projects'] }),
  })
}

// Ouvre un projet : stocke dans Pinia + navigue vers /translate
export function useOpenProject() {
  const router = useRouter()
  const projectStore = useProjectStore()

  return (project: Project) => {
    projectStore.setProject(project)
    router.push(`/projects/${project.id}/translate`)
  }
}

export function useProjectProgress(projectId: number) {
  return useQuery({
    queryKey: ['project-progress', projectId],
    queryFn: () =>
      invoke<{ done: number; total: number }>('get_project_progress', { projectId }),
    refetchInterval: 5_000,
  })
}
