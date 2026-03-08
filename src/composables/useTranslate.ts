import { computed, toRef, type MaybeRef } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query'

// ─── Types ───────────────────────────────────────────────────────────────────

export interface FileEntry {
  id: number
  relative_path: string
  strings_total: number
  strings_done: number
}

export interface StringEntry {
  id: number
  source_hash: string
  source_text: string
  raw_text: string
  context_path: string
  event_code: number | null
  row_index: number
  translation: string | null
  status: 'pending' | 'translated' | 'reviewed'
  file_path: string
}

// ─── Composables ─────────────────────────────────────────────────────────────

/** Liste les fichiers d'un projet avec leur progression */
export function useProjectFiles(projectId: number) {
  return useQuery({
    queryKey: ['project-files', projectId],
    queryFn: () => invoke<FileEntry[]>('list_project_files', { projectId }),
    enabled: projectId > 0,
  })
}

/** Retourne les strings d'un projet, filtrés de façon réactive par fichier */
export function useProjectStrings(
  projectId: number,
  filePath: MaybeRef<string | undefined> = undefined,
) {
  const filePathRef = toRef(filePath)
  return useQuery({
    queryKey: computed(() => ['project-strings', projectId, filePathRef.value ?? null]),
    queryFn: async () => {
      const all = await invoke<StringEntry[]>('get_project_strings', {
        projectId,
        statusFilter: null,
      })
      return filePathRef.value
        ? all.filter((s) => s.file_path === filePathRef.value)
        : all
    },
    enabled: projectId > 0,
  })
}

/** Extraction RPG Maker MV/MZ : lance extract_rpgmv et invalide les caches */
export function useExtractProject() {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: ({
      projectId,
      gamePath,
      workPath,
    }: {
      projectId: number
      gamePath: string
      workPath: string
    }) => invoke<number>('extract_rpgmv', { projectId, gamePath, workPath }),
    onSuccess: (_count, { projectId }) => {
      qc.invalidateQueries({ queryKey: ['project-files', projectId] })
      qc.invalidateQueries({ queryKey: ['project-strings', projectId] })
      qc.invalidateQueries({ queryKey: ['project-progress', projectId] })
      qc.invalidateQueries({ queryKey: ['projects'] })
    },
  })
}
