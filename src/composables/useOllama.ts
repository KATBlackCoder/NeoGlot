import { invoke } from '@tauri-apps/api/core'
import { useQuery } from '@tanstack/vue-query'

// Vérifie si Ollama est accessible sur localhost:11434
export function useOllamaStatus() {
  return useQuery({
    queryKey: ['ollama-status'],
    queryFn: () => invoke<boolean>('check_ollama'),
    retry: false,
    refetchInterval: 30_000,
  })
}

// Récupère la liste des modèles disponibles dans Ollama
export function useOllamaModels() {
  return useQuery({
    queryKey: ['ollama-models'],
    queryFn: () => invoke<string[]>('list_ollama_models'),
    retry: false,
  })
}
