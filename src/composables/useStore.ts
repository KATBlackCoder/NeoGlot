import { LazyStore } from '@tauri-apps/plugin-store'
import { shallowRef, readonly } from 'vue'

// Clés de configuration persistées via tauri-plugin-store
// LazyStore charge le store à la première utilisation, sans nécessiter de defaults
const store = new LazyStore('settings.json')

export interface AppSettings {
  ollama_url: string
  ollama_model: string
  tokens_per_batch: number
  source_lang: string
  target_lang: string
}

const DEFAULT_SETTINGS: AppSettings = {
  ollama_url: 'http://localhost:11434',
  ollama_model: '',
  tokens_per_batch: 2048,
  source_lang: 'ja',
  target_lang: 'fr',
}

const _settings = shallowRef<AppSettings>({ ...DEFAULT_SETTINGS })

// Charge les paramètres depuis le store Tauri
async function loadSettings() {
  try {
    const saved = await store.get<AppSettings>('settings')
    if (saved) {
      _settings.value = { ...DEFAULT_SETTINGS, ...saved }
    }
  } catch {
    // Fichier absent au premier lancement — utiliser les valeurs par défaut
  }
}

// Sauvegarde les paramètres dans le store Tauri
async function saveSettings(partial: Partial<AppSettings>) {
  _settings.value = { ..._settings.value, ...partial }
  try {
    await store.set('settings', _settings.value)
    await store.save()
  } catch (err) {
    console.error('Erreur sauvegarde settings:', err)
  }
}

export function useStore() {
  return {
    settings: readonly(_settings),
    loadSettings,
    saveSettings,
  }
}
