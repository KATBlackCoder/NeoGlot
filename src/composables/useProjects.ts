import { invoke } from '@tauri-apps/api/core'
import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query'
import { useRouter } from 'vue-router'
import type { Project } from '@/types/project'
import { useProjectStore } from '@/stores'

// ENGINE_LABELS est dans types/project.ts — ré-exporté pour rétrocompat
export { ENGINE_LABELS } from '@/types/project'

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
