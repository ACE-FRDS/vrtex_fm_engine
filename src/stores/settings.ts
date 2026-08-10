import { ref, watch } from 'vue'
import { defineStore } from 'pinia'

type SavedCodexSettings = {
  integration?: 'openai' | 'codex'
  model?: string
  authMethod?: 'chatgpt' | 'api-key'
  credentialStore?: 'environment' | 'keyring' | 'auto'
  ragEnabled?: boolean
  requireDiffReview?: boolean
}

const CODEX_SETTINGS_KEY = 'vertex.codexSettings'
const GENERAL_SETTINGS_KEY = 'vertex.generalSettings'

type SavedGeneralSettings = {
  fontSize?: number
  minimap?: boolean
  fileMakerVersion?: string
  polling?: boolean
}

function loadGeneralSettings(): SavedGeneralSettings {
  try {
    return JSON.parse(localStorage.getItem(GENERAL_SETTINGS_KEY) ?? '{}') as SavedGeneralSettings
  } catch {
    return {}
  }
}

function loadCodexSettings(): SavedCodexSettings {
  try {
    return JSON.parse(localStorage.getItem(CODEX_SETTINGS_KEY) ?? '{}') as SavedCodexSettings
  } catch {
    return {}
  }
}

export const useSettingsStore = defineStore('settings', () => {
  const savedCodex = loadCodexSettings()
  const savedGeneral = loadGeneralSettings()
  const fontSize = ref(savedGeneral.fontSize ?? 13)
  const minimap = ref(savedGeneral.minimap ?? true)
  const fileMakerVersion = ref(savedGeneral.fileMakerVersion ?? '26.0')
  const polling = ref(savedGeneral.polling ?? false)
  const codexIntegration = ref<'openai' | 'codex'>(savedCodex.integration ?? 'openai')
  const codexModel = ref(savedCodex.model ?? 'gpt-5.6-terra')
  const codexAuthMethod = ref<'chatgpt' | 'api-key'>(savedCodex.authMethod ?? 'api-key')
  const codexCredentialStore = ref<'environment' | 'keyring' | 'auto'>(savedCodex.credentialStore ?? 'environment')
  const codexRagEnabled = ref(savedCodex.ragEnabled ?? true)
  const codexRequireDiffReview = ref(savedCodex.requireDiffReview ?? true)

  watch(
    [codexIntegration, codexModel, codexAuthMethod, codexCredentialStore, codexRagEnabled, codexRequireDiffReview],
    ([integration, model, authMethod, credentialStore, ragEnabled, requireDiffReview]) => {
      localStorage.setItem(CODEX_SETTINGS_KEY, JSON.stringify({ integration, model, authMethod, credentialStore, ragEnabled, requireDiffReview }))
    },
  )

  watch(
    [fontSize, minimap, fileMakerVersion, polling],
    ([savedFontSize, savedMinimap, savedFileMakerVersion, savedPolling]) => {
      localStorage.setItem(GENERAL_SETTINGS_KEY, JSON.stringify({
        fontSize: savedFontSize,
        minimap: savedMinimap,
        fileMakerVersion: savedFileMakerVersion,
        polling: savedPolling,
      }))
    },
  )

  return {
    fontSize,
    minimap,
    fileMakerVersion,
    polling,
    codexIntegration,
    codexModel,
    codexAuthMethod,
    codexCredentialStore,
    codexRagEnabled,
    codexRequireDiffReview,
  }
})
