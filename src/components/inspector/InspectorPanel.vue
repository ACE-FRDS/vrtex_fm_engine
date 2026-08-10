<script setup lang="ts">
import { computed, ref } from 'vue'
import { useQuasar } from 'quasar'
import { sampleInspector } from '../../data/sample'
import { clipboardGateway } from '../../services/clipboardGateway'
import { isTauriRuntime, nativeGateway } from '../../services/nativeGateway'
import { formatXmlForDisplay } from '../../utils/xmlFormat'
import { useClipboardStore } from '../../stores/clipboard'
import { useEditorStore } from '../../stores/editor'
import { useLocaleStore } from '../../stores/locale'
import ValidationPanel from './ValidationPanel.vue'

const $q = useQuasar()
const clipboard = useClipboardStore()
const editor = useEditorStore()
const locale = useLocaleStore()
const sending = ref(false)
const receiving = ref(false)
let dismissTransferNotification: (() => void) | undefined

const byteSize = computed(() => new TextEncoder().encode(editor.content).byteLength)
const validationErrorCount = computed(() =>
  editor.validation.filter((result) => result.level === 'error').length,
)
const canSendToFileMaker = computed(() =>
  Boolean(clipboard.selectedItem)
  && !sending.value
  && !editor.validationPending
  && validationErrorCount.value === 0,
)
const objectSummary = computed(() => {
  const document = new DOMParser().parseFromString(editor.content, 'application/xml')
  if (document.querySelector('parsererror')) return '—'
  const scripts = document.querySelectorAll('Script').length
  const steps = document.querySelectorAll('Step').length
  const countLabel = (count: number, label: string) => `${count} ${label}${count === 1 ? '' : 's'}`
  if (scripts) return `${countLabel(scripts, 'Script')} / ${countLabel(steps, 'Step')}`
  if (steps) return countLabel(steps, 'Step')
  const objectType = clipboard.selectedItem?.objectType ?? sampleInspector.objectType
  const selectors: Record<string, string> = {
    Table: 'BaseTable, Table',
    Field: 'Field',
    Layout: 'Layout, Object',
  }
  const count = document.querySelectorAll(selectors[objectType] ?? '*').length
  return countLabel(count, objectType)
})
const details = computed(() => [
  [locale.t('formatWindows'), clipboard.selectedItem?.windowsFormat ?? sampleInspector.windowsFormat],
  [locale.t('formatInternal'), clipboard.selectedItem?.format ?? sampleInspector.internalFormat],
  [locale.t('type'), clipboard.selectedItem?.objectType ?? sampleInspector.objectType],
  [locale.t('objects'), objectSummary.value],
  [locale.t('fileMakerVersion'), sampleInspector.fileMakerVersion],
  [locale.t('size'), `${byteSize.value.toLocaleString()} bytes`],
  [locale.t('encoding'), sampleInspector.encoding],
  [locale.t('header'), `${sampleInspector.headerBytes} bytes · Little Endian`],
])

function transferCaption(name: string, format: string, bytes: number) {
  return `${name} · ${format} · ${bytes.toLocaleString()} bytes`
}

function errorCaption(error: unknown) {
  const message = error instanceof Error ? error.message : String(error)
  return message.replace(/^FileMaker Clipboard is not available:\s*/i, '')
}

function notifyTransfer(
  kind: 'get-success' | 'set-success' | 'get-error' | 'set-error',
  caption: string,
) {
  dismissTransferNotification?.()
  const success = kind.endsWith('success')
  const getting = kind.startsWith('get')
  dismissTransferNotification = $q.notify({
    position: 'bottom',
    timeout: success ? 3200 : 5600,
    progress: true,
    multiLine: true,
    icon: success ? (getting ? 'content_paste_go' : 'outbox') : 'error_outline',
    iconColor: success ? 'light-blue-4' : 'red-4',
    textColor: 'white',
    message: locale.t(
      success
        ? getting ? 'clipboardGetComplete' : 'clipboardSetComplete'
        : getting ? 'clipboardGetFailed' : 'clipboardSetFailed',
    ),
    caption,
    classes: `vertex-transfer-notification ${success ? 'transfer-success' : 'transfer-error'}`,
  })
}

async function sendToFileMaker() {
  if (!clipboard.selectedItem) {
    notifyTransfer('set-error', locale.t('clipboardNoSelection'))
    return
  }
  if (!isTauriRuntime()) {
    notifyTransfer('set-error', locale.t('desktopRequired'))
    return
  }
  sending.value = true
  try {
    const report = await nativeGateway.validateXml(editor.content, clipboard.selectedItem.format)
    editor.validation = report.issues.map(({ level, message }) => ({ level, message }))
    if (!report.valid) {
      notifyTransfer(
        'set-error',
        `${validationErrorCount.value}${locale.t('validationErrorsSuffix')}`,
      )
      return
    }
    await clipboardGateway.set(clipboard.selectedItem.format, editor.content)
    notifyTransfer(
      'set-success',
      transferCaption(clipboard.selectedItem.name, clipboard.selectedItem.format, byteSize.value),
    )
  } catch (error) {
    notifyTransfer('set-error', errorCaption(error))
  } finally {
    sending.value = false
  }
}

function itemName(xml: string, objectType: string) {
  const document = new DOMParser().parseFromString(xml, 'application/xml')
  const named = document.querySelector('Script[name], Step[name], BaseTable[name], Table[name], Field[name], Layout[name], CustomFunction[name], Function[name], Theme[name]')
  return named?.getAttribute('name')?.trim() || `${objectType} ${new Date().toLocaleString()}`
}

async function getFromFileMaker() {
  if (!isTauriRuntime()) {
    notifyTransfer('get-error', locale.t('desktopRequired'))
    return
  }
  receiving.value = true
  try {
    const payload = await clipboardGateway.get()
    const detected = await nativeGateway.detectFormat(payload.xml)
    const format = detected.format === 'UNKNOWN' ? payload.format : detected.format
    const saved = await clipboard.upsert({
      name: itemName(payload.xml, detected.objectType),
      format,
      windowsFormat: payload.windowsFormat,
      objectType: detected.objectType,
      xml: payload.xml,
      notes: '',
      favorite: false,
      inLibrary: false,
    })
    const displayXml = formatXmlForDisplay(saved.xml)
    editor.content = displayXml
    editor.savedContent = displayXml
    await Promise.all([editor.validate(saved.format), editor.buildPreview()])
    notifyTransfer(
      'get-success',
      transferCaption(saved.name, saved.format, payload.rawSize),
    )
  } catch (error) {
    notifyTransfer('get-error', errorCaption(error))
  } finally {
    receiving.value = false
  }
}

async function validate() {
  await editor.validate(clipboard.selectedItem?.format)
  const errorCount = editor.validation.filter((result) => result.level === 'error').length
  $q.notify({
    type: errorCount ? 'negative' : 'positive',
    message: errorCount ? `${errorCount}${locale.t('validationErrorsSuffix')}` : locale.t('validationPassedNotice'),
  })
}

async function save() {
  const item = clipboard.selectedItem
  if (!item) return
  try {
    await clipboard.upsert({
      id: item.id,
      name: item.name,
      format: item.format,
      windowsFormat: item.windowsFormat,
      objectType: item.objectType,
      xml: editor.content,
      notes: item.notes,
      favorite: item.favorite,
      inLibrary: true,
    })
    editor.save()
    $q.notify({ type: 'positive', message: locale.t('savedClipboardItem') })
  } catch (error) {
    $q.notify({ type: 'negative', message: String(error) })
  }
}

async function toggleFavorite() {
  try {
    await clipboard.toggleFavorite()
  } catch (error) {
    $q.notify({ type: 'negative', message: String(error) })
  }
}

async function copyAsText() {
  await navigator.clipboard.writeText(editor.content)
  $q.notify({ type: 'positive', message: locale.t('copiedXmlText') })
}
</script>

<template>
  <aside class="inspector-panel">
    <div class="inspector-title">
      <div><span class="eyebrow">{{ locale.t('analyze') }}</span><h2>{{ locale.t('inspector') }}</h2></div>
      <q-btn flat dense round size="sm" icon="tune" />
    </div>

    <section class="inspector-section details-section">
      <div class="section-heading"><span>{{ locale.t('objectDetails') }}</span><i /></div>
      <dl>
        <template v-for="[label, value] in details" :key="label">
          <dt>{{ label }}</dt>
          <dd :class="{ accent: label === locale.t('formatInternal') || label === locale.t('type') }">{{ value }}</dd>
        </template>
      </dl>
    </section>

    <section class="inspector-section validation-section">
      <div class="section-heading">
        <span>{{ locale.t('validation') }}</span>
        <q-badge
          :color="validationErrorCount ? 'negative' : 'positive'"
          :text-color="validationErrorCount ? 'white' : 'dark'"
          :label="validationErrorCount ? `${validationErrorCount}${locale.t('validationErrorsSuffix')}` : locale.t('pass')"
        />
      </div>
      <ValidationPanel :results="editor.validation" />
      <button class="secondary-action" type="button" @click="validate">
        <span class="material-icons">fact_check</span> {{ locale.t('validateAgain') }}
      </button>
    </section>

    <section class="inspector-section action-section">
      <div class="section-heading"><span>{{ locale.t('actions') }}</span><i /></div>
      <button class="get-button" type="button" :disabled="receiving" @click="getFromFileMaker">
        <span class="material-icons">content_paste_go</span>
        <strong>{{ receiving ? locale.t('receiving') : locale.t('getFromFileMaker') }}</strong>
      </button>
      <button class="send-button" type="button" :disabled="!canSendToFileMaker" @click="sendToFileMaker">
        <span class="send-icon material-icons">send</span>
        <span><strong>{{ sending ? locale.t('sending') : locale.t('sendToFileMaker') }}</strong><small>{{ locale.t('writeXmlClipboard') }}</small></span>
        <span class="material-icons arrow">arrow_forward</span>
      </button>
      <div class="action-grid">
        <button type="button" @click="save"><span class="material-icons">save</span>{{ locale.t('save') }}</button>
        <button type="button" @click="toggleFavorite">
          <span class="material-icons">{{ clipboard.selectedItem?.favorite ? 'star' : 'star_border' }}</span>{{ locale.t('favorite') }}
        </button>
        <button type="button"><span class="material-icons">download</span>{{ locale.t('saveFile') }}</button>
        <button type="button" @click="copyAsText"><span class="material-icons">content_copy</span>{{ locale.t('copyText') }}</button>
      </div>
    </section>
  </aside>
</template>
