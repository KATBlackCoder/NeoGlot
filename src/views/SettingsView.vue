<script setup lang="ts">
import { shallowRef, onMounted } from 'vue'
import { SettingsIcon } from 'lucide-vue-next'
import { useStore } from '@/composables/useStore'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Separator } from '@/components/ui/separator'
import { useOllamaModels } from '@/composables/useOllama'

const { settings, loadSettings, saveSettings } = useStore()
const { data: models, isPending: modelsLoading } = useOllamaModels()

// Champs locaux pour la saisie — synchronisés au montage depuis le store
const ollamaUrl = shallowRef('')
const ollamaModel = shallowRef('')
const tokensPerBatch = shallowRef(2048)

onMounted(async () => {
  await loadSettings()
  ollamaUrl.value = settings.value.ollama_url
  ollamaModel.value = settings.value.ollama_model
  tokensPerBatch.value = settings.value.tokens_per_batch
})

async function handleSave() {
  await saveSettings({
    ollama_url: ollamaUrl.value,
    ollama_model: ollamaModel.value,
    tokens_per_batch: tokensPerBatch.value,
  })
}
</script>

<template>
  <div class="p-6 max-w-2xl space-y-8">
    <div class="flex items-center gap-2">
      <SettingsIcon class="size-5 text-muted-foreground" />
      <h1 class="text-2xl font-bold tracking-tight">Paramètres</h1>
    </div>

    <!-- Configuration Ollama -->
    <section class="space-y-4">
      <div>
        <h2 class="text-base font-semibold">Ollama</h2>
        <p class="text-sm text-muted-foreground">Configuration du moteur IA local.</p>
      </div>

      <div class="space-y-3">
        <div class="space-y-1.5">
          <label class="text-sm font-medium" for="ollama-url">URL Ollama</label>
          <Input
            id="ollama-url"
            v-model="ollamaUrl"
            placeholder="http://localhost:11434"
          />
        </div>

        <div class="space-y-1.5">
          <label class="text-sm font-medium" for="ollama-model">Modèle</label>
          <Select v-model="ollamaModel">
            <SelectTrigger id="ollama-model">
              <SelectValue :placeholder="modelsLoading ? 'Chargement...' : 'Choisir un modèle'" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem
                v-for="model in (models ?? [])"
                :key="model"
                :value="model"
              >
                {{ model }}
              </SelectItem>
              <SelectItem v-if="!modelsLoading && !models?.length" value="" disabled>
                Aucun modèle disponible
              </SelectItem>
            </SelectContent>
          </Select>
        </div>
      </div>
    </section>

    <Separator />

    <!-- Configuration traduction -->
    <section class="space-y-4">
      <div>
        <h2 class="text-base font-semibold">Traduction</h2>
        <p class="text-sm text-muted-foreground">Paramètres du pipeline de traduction.</p>
      </div>

      <div class="space-y-1.5">
        <label class="text-sm font-medium" for="tokens-batch">Tokens par batch</label>
        <Input
          id="tokens-batch"
          type="number"
          :model-value="tokensPerBatch"
          @update:model-value="tokensPerBatch = Number($event)"
          min="512"
          max="8192"
          step="256"
        />
        <p class="text-xs text-muted-foreground">
          Nombre de tokens maximum envoyés à Ollama par requête (recommandé : 2048).
        </p>
      </div>
    </section>

    <Separator />

    <Button @click="handleSave">Sauvegarder</Button>
  </div>
</template>
