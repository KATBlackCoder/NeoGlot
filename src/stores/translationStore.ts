import { defineStore } from 'pinia'
import { shallowRef, computed } from 'vue'

// Représente un event de progression reçu via Tauri Channel
export interface TranslationProgress {
  done: number
  total: number
  last_translation: string
  status: 'running' | 'done' | 'cancelled' | 'error'
}

// Store du job de traduction en cours
// Partagé entre TranslateView (boutons Pause/Traduire) et la ProgressBar globale
export const useTranslationStore = defineStore('translation', () => {
  const isRunning = shallowRef(false)
  const progress = shallowRef<TranslationProgress | null>(null)

  const percentage = computed(() => {
    if (!progress.value || progress.value.total === 0) return 0
    return Math.round((progress.value.done / progress.value.total) * 100)
  })

  function start() {
    isRunning.value = true
    progress.value = null
  }

  function updateProgress(event: TranslationProgress) {
    progress.value = event
    if (event.status !== 'running') {
      isRunning.value = false
    }
  }

  function stop() {
    isRunning.value = false
  }

  function reset() {
    isRunning.value = false
    progress.value = null
  }

  return {
    isRunning,
    progress,
    percentage,
    start,
    updateProgress,
    stop,
    reset,
  }
})
