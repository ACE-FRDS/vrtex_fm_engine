<script setup lang="ts">
import { computed, ref } from 'vue'
import { useQuasar } from 'quasar'
import { useClipboardStore } from '../../stores/clipboard'
import { useEditorStore } from '../../stores/editor'
import { useLibraryStore } from '../../stores/library'
import { useSettingsStore } from '../../stores/settings'
import { useLocaleStore } from '../../stores/locale'
import { useAiAssistantStore } from '../../stores/aiAssistant'
import { aiGateway } from '../../services/aiGateway'
import { isTauriRuntime } from '../../services/nativeGateway'
import type { AiConnectionTest } from '../../types/ai'
import type { WorkspaceMode } from '../../stores/navigation'

const props = defineProps<{ mode: Exclude<WorkspaceMode, 'clipboard' | 'codex' | 'docs'> }>()
const $q = useQuasar()
const clipboard = useClipboardStore()
const editor = useEditorStore()
const library = useLibraryStore()
const settings = useSettingsStore()
const locale = useLocaleStore()
const ai = useAiAssistantStore()
const settingsSection = ref<'general' | 'codex' | 'data'>('general')
const openAiApiKey = ref('')
const credentialBusy = ref(false)
const connectionTest = ref<AiConnectionTest | null>(null)

const selectedProviderStatus = computed(() =>
  ai.providers.find((provider) => provider.id === settings.codexIntegration) ?? null,
)

const providerStatusLabel = computed(() => {
  if (connectionTest.value?.provider === settings.codexIntegration) {
    return connectionTest.value.success ? '接続確認済み' : '接続失敗'
  }
  return selectedProviderStatus.value?.authenticated ? '認証情報あり' : '未接続'
})

const headings = computed(() => ({
  library: {
    eyebrow: locale.t('libraryEyebrow'),
    title: locale.t('navLibrary'),
    description: locale.t('libraryDescription'),
    icon: 'library_books',
  },
  collections: {
    eyebrow: locale.t('collectionsEyebrow'),
    title: locale.t('navCollections'),
    description: locale.t('collectionsDescription'),
    icon: 'account_tree',
  },
  tools: {
    eyebrow: locale.t('toolsEyebrow'),
    title: locale.t('navTools'),
    description: locale.t('toolsDescription'),
    icon: 'construction',
  },
  settings: {
    eyebrow: locale.t('settingsEyebrow'),
    title: locale.t('navSettings'),
    description: locale.t('settingsDescription'),
    icon: 'settings',
  },
} as const))

const heading = computed(() => headings.value[props.mode])

const tools = computed(() => [
  { icon: 'fact_check', name: 'XML Validation', detail: locale.t('toolXmlValidationDescription'), state: locale.t('available'), planned: false },
  { icon: 'radar', name: 'Format Detection', detail: locale.t('toolFormatDetectionDescription'), state: locale.t('available'), planned: false },
  { icon: 'difference', name: 'XML Diff', detail: locale.t('toolDiffDescription'), state: locale.t('available'), planned: false },
  { icon: 'account_tree', name: 'Structure Viewer', detail: locale.t('toolStructureDescription'), state: locale.t('available'), planned: false },
  { icon: 'build_circle', name: 'XML Repair', detail: locale.t('toolRepairDescription'), state: locale.t('planned'), planned: true },
  { icon: 'psychology', name: 'AI Assistant', detail: locale.t('toolAiDescription'), state: locale.t('available'), planned: false },
])

type DeleteScope = 'library' | 'clipboard' | 'all'

function confirmDeletion(scope: DeleteScope) {
  const labels = {
    library: locale.t('deleteLibrary'),
    clipboard: locale.t('deleteClipboard'),
    all: locale.t('deleteAll'),
  }
  const count = scope === 'library'
    ? clipboard.libraryItems.length
    : scope === 'clipboard'
      ? clipboard.items.length
      : clipboard.libraryItems.length + clipboard.items.length
  const message = locale.language === 'ja'
    ? `${labels[scope]}を実行します。対象は${count}件です。この操作は取り消せません。`
    : `${labels[scope]} will remove ${count} item(s). This action cannot be undone.`

  $q.dialog({
    title: locale.t('deleteConfirmTitle'),
    message,
    persistent: true,
    cancel: { label: locale.t('cancel'), flat: true, color: 'grey-5' },
    ok: { label: locale.t('deleteAction'), color: 'negative', unelevated: true },
  }).onOk(async () => {
    try {
      if (scope === 'all') {
        await clipboard.clearAll()
        library.clearCollections()
      } else if (scope === 'library') {
        await clipboard.clearLibrary()
        library.clearCollections()
      } else {
        await clipboard.clearClipboard()
      }
      if (scope === 'clipboard' || scope === 'all') {
        editor.content = ''
        editor.savedContent = ''
        editor.activeTab = 'xml'
      }
      $q.notify({ type: 'positive', message: locale.t('deleteCompleted') })
    } catch (error) {
      $q.notify({ type: 'negative', message: String(error) })
    }
  })
}

async function checkCodexConnection() {
  if (!isTauriRuntime()) {
    $q.notify({ type: 'warning', icon: 'smart_toy', message: '接続確認はデスクトップアプリで実行してください' })
    return
  }
  credentialBusy.value = true
  try {
    connectionTest.value = await aiGateway.testProviderConnection(settings.codexIntegration)
    await ai.refreshProviders()
    $q.notify({
      type: connectionTest.value.success ? 'positive' : 'negative',
      icon: connectionTest.value.success ? 'cloud_done' : 'cloud_off',
      message: connectionTest.value.detail,
      caption: connectionTest.value.requestId ? `Request ID: ${connectionTest.value.requestId}` : undefined,
    })
  } catch (error) {
    connectionTest.value = {
      provider: settings.codexIntegration,
      success: false,
      detail: String(error),
      requestId: null,
    }
    $q.notify({ type: 'negative', icon: 'cloud_off', message: String(error) })
  } finally {
    credentialBusy.value = false
  }
}

async function saveOpenAiApiKey() {
  if (!openAiApiKey.value.trim() || credentialBusy.value) return
  credentialBusy.value = true
  try {
    await aiGateway.saveOpenAiApiKey(openAiApiKey.value)
    openAiApiKey.value = ''
    settings.codexAuthMethod = 'api-key'
    settings.codexCredentialStore = 'keyring'
    connectionTest.value = null
    await ai.refreshProviders()
    $q.notify({ type: 'positive', icon: 'verified_user', message: 'APIキーをWindows保護ストレージへ保存しました' })
  } catch (error) {
    $q.notify({ type: 'negative', message: String(error) })
  } finally {
    credentialBusy.value = false
  }
}

function confirmOpenAiApiKeyDeletion() {
  $q.dialog({
    title: '保存済みAPIキーを削除しますか？',
    message: 'Windows保護ストレージに保存したOpenAI APIキーを削除します。環境変数は変更しません。',
    persistent: true,
    cancel: { label: locale.t('cancel'), flat: true, color: 'grey-5' },
    ok: { label: 'APIキーを削除', color: 'negative', unelevated: true },
  }).onOk(async () => {
    credentialBusy.value = true
    try {
      const deleted = await aiGateway.deleteOpenAiApiKey()
      connectionTest.value = null
      await ai.refreshProviders()
      $q.notify({
        type: deleted ? 'positive' : 'info',
        message: deleted ? '保存済みAPIキーを削除しました' : '保存済みAPIキーはありません',
      })
    } catch (error) {
      $q.notify({ type: 'negative', message: String(error) })
    } finally {
      credentialBusy.value = false
    }
  })
}
</script>

<template>
  <main class="module-workspace">
    <header class="module-header">
      <div class="module-icon"><span class="material-icons">{{ heading.icon }}</span></div>
      <div class="module-header-copy">
        <span class="module-header-eyebrow">{{ heading.eyebrow }}</span>
        <h1>{{ heading.title }}</h1>
        <p>{{ heading.description }}</p>
      </div>
    </header>

    <section v-if="mode === 'library'" class="module-body library-workspace">
      <div class="module-toolbar">
        <label><span class="material-icons">search</span><input type="search" :placeholder="locale.t('searchLibrary')" /></label>
        <button type="button"><span class="material-icons">sort</span>{{ locale.t('sortRecent') }}</button>
      </div>
      <div class="library-grid">
        <div v-if="clipboard.libraryItems.length === 0" class="library-empty">
          <span class="material-icons">inventory_2</span>
          <p>{{ locale.t('noItems') }}</p>
        </div>
        <article v-for="item in clipboard.libraryItems" :key="item.id" class="library-card">
          <div class="library-format">{{ item.format }}</div>
          <button
            class="library-star"
            :class="{ active: item.favorite }"
            type="button"
            :aria-label="item.favorite ? locale.t('removeFavorite') : locale.t('addFavorite')"
            @click="clipboard.toggleFavorite(item.id)"
          >
            <span class="material-icons">{{ item.favorite ? 'star' : 'star_border' }}</span>
          </button>
          <strong>{{ item.name }}</strong>
          <small>{{ item.objectType }} · {{ item.windowsFormat }}</small>
          <div><span v-for="tag in item.tags" :key="tag">{{ tag }}</span></div>
        </article>
      </div>
    </section>

    <section v-else-if="mode === 'collections'" class="module-body collections-workspace">
      <div class="module-toolbar">
        <strong>{{ library.collections.length }} {{ locale.t('projectCollections') }}</strong>
        <button type="button"><span class="material-icons">create_new_folder</span>{{ locale.t('newCollection') }}</button>
      </div>
      <div class="collection-board">
        <article v-for="collection in library.collections" :key="collection.id" class="collection-board-card">
          <header><span class="material-icons">folder_open</span><strong>{{ collection.name }}</strong><em>{{ collection.count }}</em></header>
          <div v-for="child in collection.children" :key="child.id">
            <span class="material-icons">folder</span><span>{{ child.name }}</span><em>{{ child.count }}</em>
          </div>
        </article>
      </div>
    </section>

    <section v-else-if="mode === 'tools'" class="module-body tools-grid">
      <article v-for="tool in tools" :key="tool.name" class="tool-card" :class="{ planned: tool.planned }">
        <span class="material-icons">{{ tool.icon }}</span>
        <div class="tool-card-copy">
          <strong>{{ tool.name }}</strong>
          <div class="tool-description">
            <span>{{ locale.t('descriptionLabel') }}</span>
            <p>{{ tool.detail }}</p>
          </div>
        </div>
        <small>{{ tool.state }}</small>
      </article>
    </section>

    <section v-else class="module-body settings-workspace">
      <nav class="settings-subnav" :aria-label="locale.t('navSettings')">
        <button type="button" :class="{ active: settingsSection === 'general' }" @click="settingsSection = 'general'"><span class="settings-tab-content"><span class="material-icons">tune</span><span>{{ locale.t('settingsGeneralTab') }}</span></span></button>
        <button type="button" :class="{ active: settingsSection === 'codex' }" @click="settingsSection = 'codex'"><span class="settings-tab-content"><span class="material-icons">smart_toy</span><span>{{ locale.t('settingsCodexTab') }}</span></span></button>
        <button type="button" :class="{ active: settingsSection === 'data' }" @click="settingsSection = 'data'"><span class="settings-tab-content"><span class="material-icons">database</span><span>{{ locale.t('settingsDataTab') }}</span></span></button>
      </nav>

      <Transition name="settings-slide" mode="out-in">
        <div v-if="settingsSection === 'general'" key="general" class="settings-page general-settings-page">
          <div class="settings-group">
            <span>{{ locale.t('editorSettings') }}</span>
            <label><div><strong>{{ locale.t('monacoFontSize') }}</strong><small>{{ locale.t('monacoFontSizeHelp') }}</small></div><input v-model.number="settings.fontSize" type="number" min="10" max="24" /></label>
            <label><div><strong>{{ locale.t('minimap') }}</strong><small>{{ locale.t('minimapHelp') }}</small></div><input v-model="settings.minimap" type="checkbox" role="switch" :aria-label="locale.t('minimap')" /></label>
          </div>
          <div class="settings-group">
            <span>{{ locale.t('fileMakerSettings') }}</span>
            <label><div><strong>{{ locale.t('defaultVersion') }}</strong><small>{{ locale.t('defaultVersionHelp') }}</small></div><select v-model="settings.fileMakerVersion"><option>26.0</option><option>25.0</option><option>24.0</option></select></label>
            <label><div><strong>{{ locale.t('clipboardPolling') }}</strong><small>{{ locale.t('clipboardPollingHelp') }}</small></div><input v-model="settings.polling" type="checkbox" role="switch" :aria-label="locale.t('clipboardPolling')" /></label>
          </div>
          <div class="settings-group">
            <span>{{ locale.t('languageSettings') }}</span>
            <label>
              <div><strong>{{ locale.t('language') }}</strong><small>{{ locale.t('languageHelp') }}</small></div>
              <select v-model="locale.language" :aria-label="locale.t('language')">
                <option value="ja">{{ locale.t('japanese') }}</option>
                <option value="en">{{ locale.t('english') }}</option>
              </select>
            </label>
          </div>
        </div>

        <div v-else-if="settingsSection === 'codex'" key="codex" class="settings-page codex-settings-page">
          <div class="settings-group codex-settings-group">
            <span>{{ locale.t('codexSettings') }}</span>
            <div class="codex-settings-intro">
              <div class="codex-settings-icon"><span class="material-icons">smart_toy</span></div>
              <div>
                <strong>{{ locale.t('codexConnectionTitle') }}</strong>
                <small>{{ locale.t('codexConnectionHelp') }}</small>
              </div>
              <em :class="{ online: selectedProviderStatus?.authenticated, failed: connectionTest && !connectionTest.success }"><i />{{ providerStatusLabel }}</em>
            </div>
            <div class="codex-settings-grid">
              <label>
                <div><strong>{{ locale.t('codexIntegrationMethod') }}</strong><small>{{ locale.t('codexIntegrationMethodHelp') }}</small></div>
                <select v-model="settings.codexIntegration" :aria-label="locale.t('codexIntegrationMethod')"><option value="openai">OpenAI Responses API</option><option value="codex">Codex App Server（Phase 2）</option></select>
              </label>
              <label>
                <div><strong>AI Model</strong><small>AI Assistantで使用するモデル</small></div>
                <select v-model="settings.codexModel" aria-label="AI Model"><option value="gpt-5.6-terra">gpt-5.6-terra</option><option value="gpt-5.6-sol">gpt-5.6-sol</option></select>
              </label>
              <label>
                <div><strong>{{ locale.t('codexAuthMethod') }}</strong><small>{{ locale.t('codexAuthMethodHelp') }}</small></div>
                <select v-model="settings.codexAuthMethod" :aria-label="locale.t('codexAuthMethod')"><option value="api-key">{{ locale.t('signInWithApiKey') }}</option><option value="chatgpt" disabled>{{ locale.t('signInWithChatGpt') }}（Phase 2）</option></select>
              </label>
              <label>
                <div><strong>{{ locale.t('codexCredentialStore') }}</strong><small>{{ locale.t('codexCredentialStoreHelp') }}</small></div>
                <select v-model="settings.codexCredentialStore" :aria-label="locale.t('codexCredentialStore')"><option value="keyring">Windows保護ストレージ（推奨）</option><option value="environment">起動環境（OPENAI_API_KEY）</option><option value="auto">{{ locale.t('credentialStoreAuto') }}</option></select>
              </label>
              <label>
                <div><strong>{{ locale.t('codexRagSetting') }}</strong><small>{{ locale.t('codexRagSettingHelp') }}</small></div>
                <input v-model="settings.codexRagEnabled" type="checkbox" role="switch" :aria-label="locale.t('codexRagSetting')" />
              </label>
              <label>
                <div><strong>{{ locale.t('codexDiffReview') }}</strong><small>{{ locale.t('codexDiffReviewHelp') }}</small></div>
                <input v-model="settings.codexRequireDiffReview" type="checkbox" role="switch" :aria-label="locale.t('codexDiffReview')" />
              </label>
            </div>
            <div v-if="settings.codexIntegration === 'openai'" class="api-key-settings">
              <div class="api-key-copy">
                <strong>OpenAI APIキー</strong>
                <small>APIキーは現在のWindowsユーザーだけが復号できます。画面・SQLite・localStorageには保存しません。</small>
              </div>
              <label class="api-key-entry">
                <span class="material-icons">key</span>
                <input v-model="openAiApiKey" type="password" autocomplete="new-password" spellcheck="false" placeholder="sk-…" aria-label="OpenAI APIキー" @keyup.enter="saveOpenAiApiKey" />
              </label>
              <button type="button" :disabled="!openAiApiKey.trim() || credentialBusy || !isTauriRuntime()" @click="saveOpenAiApiKey"><span class="material-icons">security</span>安全に保存</button>
              <button class="delete-credential" type="button" :disabled="credentialBusy || !selectedProviderStatus?.authenticated || !isTauriRuntime()" @click="confirmOpenAiApiKeyDeletion"><span class="material-icons">delete_outline</span>保存キーを削除</button>
            </div>
            <div class="codex-setup-guide">
              <strong>{{ locale.t('codexSetupProcedure') }}</strong>
              <ol>
                <li><span>1</span><div><strong>{{ locale.t('codexSetupRuntime') }}</strong><small>{{ locale.t('codexSetupRuntimeHelp') }}</small></div></li>
                <li><span>2</span><div><strong>{{ locale.t('codexSetupLogin') }}</strong><small>{{ locale.t('codexSetupLoginHelp') }}</small></div></li>
                <li><span>3</span><div><strong>{{ locale.t('codexSetupConnect') }}</strong><small>{{ locale.t('codexSetupConnectHelp') }}</small></div></li>
              </ol>
              <p><span class="material-icons">shield</span>{{ locale.t('codexSecretNotice') }}</p>
            </div>
            <div class="codex-settings-actions">
              <span :class="{ online: connectionTest?.success, failed: connectionTest && !connectionTest.success }"><i />{{ connectionTest?.detail ?? selectedProviderStatus?.detail ?? locale.t('codexConnectionPending') }}</span>
              <button type="button" :disabled="credentialBusy" @click="checkCodexConnection"><span class="material-icons">{{ credentialBusy ? 'hourglass_top' : 'sync' }}</span>{{ credentialBusy ? '確認中…' : locale.t('checkConnection') }}</button>
            </div>
          </div>
        </div>

        <div v-else key="data" class="settings-page data-settings-page">
          <div class="settings-group danger-settings">
            <span>{{ locale.t('dataManagement') }}</span>
            <div class="danger-settings-intro"><strong>{{ locale.t('deleteStoredData') }}</strong><small>{{ locale.t('deleteStoredDataHelp') }}</small></div>
            <div class="delete-setting-row"><div><strong>{{ locale.t('deleteLibrary') }}</strong><small>{{ locale.t('deleteLibraryHelp') }}</small></div><button type="button" @click="confirmDeletion('library')">{{ locale.t('deleteLibrary') }}</button></div>
            <div class="delete-setting-row"><div><strong>{{ locale.t('deleteClipboard') }}</strong><small>{{ locale.t('deleteClipboardHelp') }}</small></div><button type="button" @click="confirmDeletion('clipboard')">{{ locale.t('deleteClipboard') }}</button></div>
            <div class="delete-setting-row critical"><div><strong>{{ locale.t('deleteAll') }}</strong><small>{{ locale.t('deleteAllHelp') }}</small></div><button type="button" @click="confirmDeletion('all')">{{ locale.t('deleteAll') }}</button></div>
          </div>
        </div>
      </Transition>
    </section>
  </main>
</template>
