<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
import * as monaco from 'monaco-editor'
import EditorWorker from 'monaco-editor/esm/vs/editor/editor.worker?worker'
import { useSettingsStore } from '../../stores/settings'

const model = defineModel<string>({ required: true })
const settings = useSettingsStore()
const container = ref<HTMLElement | null>(null)
let editor: monaco.editor.IStandaloneCodeEditor | undefined
let subscription: monaco.IDisposable | undefined

const monacoGlobal = self as typeof globalThis & {
  MonacoEnvironment?: { getWorker: () => Worker }
}
monacoGlobal.MonacoEnvironment = { getWorker: () => new EditorWorker() }

monaco.editor.defineTheme('vertex-dark', {
  base: 'vs-dark',
  inherit: true,
  rules: [
    { token: 'tag', foreground: '55B7FF' },
    { token: 'attribute.name', foreground: '8CD5FF' },
    { token: 'attribute.value', foreground: 'C8E68B' },
    { token: 'delimiter', foreground: '66849F' },
  ],
  colors: {
    'editor.background': '#0A0F16',
    'editor.foreground': '#C8D4DF',
    'editorLineNumber.foreground': '#344252',
    'editorLineNumber.activeForeground': '#8FB2D3',
    'editor.lineHighlightBackground': '#101923',
    'editor.selectionBackground': '#164A7366',
    'editorCursor.foreground': '#28A4FF',
    'editorIndentGuide.background1': '#1A2633',
    'editorIndentGuide.activeBackground1': '#37516B',
    'editorGutter.background': '#0A0F16',
  },
})

onMounted(() => {
  if (!container.value) return
  editor = monaco.editor.create(container.value, {
    value: model.value,
    language: 'xml',
    theme: 'vertex-dark',
    automaticLayout: true,
    fontFamily: "'JetBrains Mono', 'Cascadia Code', Consolas, monospace",
    fontSize: settings.fontSize,
    lineHeight: 21,
    minimap: { enabled: settings.minimap, scale: 0.8 },
    padding: { top: 14, bottom: 14 },
    renderLineHighlight: 'all',
    smoothScrolling: true,
    scrollBeyondLastLine: false,
    bracketPairColorization: { enabled: true },
    guides: { bracketPairs: true, indentation: true },
    folding: true,
    wordWrap: 'off',
  })
  subscription = editor.onDidChangeModelContent(() => {
    const value = editor?.getValue() ?? ''
    if (value !== model.value) model.value = value
  })
})

watch(model, (value) => {
  if (editor && value !== editor.getValue()) editor.setValue(value)
})

watch(
  () => [settings.fontSize, settings.minimap] as const,
  ([fontSize, minimap]) => editor?.updateOptions({ fontSize, minimap: { enabled: minimap } }),
)

onBeforeUnmount(() => {
  subscription?.dispose()
  editor?.dispose()
})
</script>

<template>
  <div ref="container" class="monaco-host" />
</template>
