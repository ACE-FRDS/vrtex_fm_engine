import { computed, ref, watch } from 'vue'
import { defineStore } from 'pinia'
import { sampleXml } from '../data/sample'
import { isTauriRuntime, nativeGateway, type ScriptPreview } from '../services/nativeGateway'
import type { ValidationResult } from '../types/clipboard'

export type EditorTab = 'xml' | 'preview' | 'structure' | 'diff' | 'notes'

export const useEditorStore = defineStore('editor', () => {
  const content = ref(sampleXml)
  const savedContent = ref(sampleXml)
  const activeTab = ref<EditorTab>('xml')
  const validation = ref<ValidationResult[]>([
    { level: 'success', message: 'XML is valid' },
    { level: 'success', message: 'Format XMSC' },
    { level: 'success', message: 'Clipboard data can be generated' },
  ])
  const preview = ref<ScriptPreview>({ name: 'Untitled Script', steps: [] })
  const validationPending = ref(false)
  let validationTimer: number | undefined

  const modified = computed(() => content.value !== savedContent.value)

  function save() {
    savedContent.value = content.value
  }

  async function validate(format?: string) {
    validationPending.value = true
    try {
      if (isTauriRuntime()) {
        const report = await nativeGateway.validateXml(content.value, format)
        validation.value = report.issues.map(({ level, message }) => ({ level, message }))
      } else {
        const parsed = new DOMParser().parseFromString(content.value, 'application/xml')
        const parseError = parsed.querySelector('parsererror')
        validation.value = parseError
          ? [{ level: 'error', message: 'XMLの構文を解析できません' }]
          : [
              { level: 'success', message: 'XMLは有効です' },
              { level: 'success', message: `形式 ${format ?? 'XML'}` },
              { level: 'success', message: 'クリップボードデータを生成できます' },
            ]
      }
    } finally {
      validationPending.value = false
    }
  }

  async function buildPreview() {
    if (isTauriRuntime()) {
      try {
        preview.value = await nativeGateway.previewXml(content.value)
        return
      } catch {
        preview.value = { name: 'Untitled Script', steps: [] }
        return
      }
    }
    const parsed = new DOMParser().parseFromString(content.value, 'application/xml')
    const script = parsed.querySelector('Script')
    preview.value = {
      name: script?.getAttribute('name') ?? 'Untitled Script',
      steps: [...parsed.querySelectorAll('Step')].map((step, index) => ({
        index: index + 1,
        name: step.getAttribute('name') ?? step.getAttribute('type') ?? 'Unknown Step',
        enabled: step.getAttribute('enable') !== 'False' && step.getAttribute('enabled') !== 'false',
      })),
    }
  }

  watch(content, (xml) => {
    window.clearTimeout(validationTimer)
    validationTimer = window.setTimeout(() => {
      void validate()
      void buildPreview()
    }, 300)
  })

  return {
    content,
    savedContent,
    activeTab,
    validation,
    preview,
    validationPending,
    modified,
    save,
    validate,
    buildPreview,
  }
})
