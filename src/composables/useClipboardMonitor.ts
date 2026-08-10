import { onBeforeUnmount, watch } from 'vue'
import { useQuasar } from 'quasar'
import { clipboardGateway } from '../services/clipboardGateway'
import { isTauriRuntime, nativeGateway } from '../services/nativeGateway'
import { useClipboardStore } from '../stores/clipboard'
import { useLocaleStore } from '../stores/locale'
import { useSettingsStore } from '../stores/settings'

const POLLING_INTERVAL_MS = 1_500

function itemName(xml: string, objectType: string) {
  const document = new DOMParser().parseFromString(xml, 'application/xml')
  const named = document.querySelector(
    'Script[name], Step[name], BaseTable[name], Table[name], Field[name], Layout[name], CustomFunction[name], Function[name], Theme[name]',
  )
  return named?.getAttribute('name')?.trim() || `${objectType} ${new Date().toLocaleString()}`
}

function isClipboardUnavailable(error: unknown) {
  return /not available|no registered Mac-XM|OpenClipboard/i.test(String(error))
}

export function useClipboardMonitor() {
  const $q = useQuasar()
  const clipboard = useClipboardStore()
  const locale = useLocaleStore()
  const settings = useSettingsStore()
  let timer: number | undefined
  let polling = false
  let lastFingerprint = ''
  let lastError = ''

  function stop() {
    window.clearTimeout(timer)
    timer = undefined
  }

  function schedule() {
    stop()
    if (settings.polling && isTauriRuntime()) {
      timer = window.setTimeout(poll, POLLING_INTERVAL_MS)
    }
  }

  async function poll() {
    if (polling || !settings.polling || !isTauriRuntime()) {
      schedule()
      return
    }

    polling = true
    try {
      const payload = await clipboardGateway.get()
      const fingerprint = `${payload.windowsFormat}\u0000${payload.xml}`
      if (fingerprint === lastFingerprint) return

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
      }, { select: false })

      lastFingerprint = fingerprint
      lastError = ''
      $q.notify({
        position: 'bottom',
        timeout: 2600,
        progress: true,
        icon: 'content_paste_go',
        iconColor: 'light-blue-4',
        textColor: 'white',
        message: locale.t('clipboardGetComplete'),
        caption: `${saved.name} · ${saved.format} · ${payload.rawSize.toLocaleString()} bytes`,
        classes: 'vertex-transfer-notification transfer-success',
      })
    } catch (error) {
      if (!isClipboardUnavailable(error)) {
        const message = String(error)
        if (message !== lastError) {
          lastError = message
          $q.notify({
            position: 'bottom',
            timeout: 4800,
            progress: true,
            icon: 'error_outline',
            iconColor: 'red-4',
            textColor: 'white',
            message: locale.t('clipboardGetFailed'),
            caption: message,
            classes: 'vertex-transfer-notification transfer-error',
          })
        }
      }
    } finally {
      polling = false
      schedule()
    }
  }

  const stopWatching = watch(
    () => settings.polling,
    (enabled) => {
      stop()
      lastError = ''
      if (enabled) schedule()
    },
    { immediate: true },
  )

  onBeforeUnmount(() => {
    stop()
    stopWatching()
  })
}
