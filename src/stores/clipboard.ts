import { computed, ref, watch } from 'vue'
import { defineStore } from 'pinia'
import { sampleHistory } from '../data/sample'
import { isTauriRuntime, nativeGateway, type NativeClipboardItem, type SaveClipboardItemInput } from '../services/nativeGateway'
import type { ClipboardItem } from '../types/clipboard'

export const useClipboardStore = defineStore('clipboard', () => {
  const loadItems = (key: string) => {
    const stored = localStorage.getItem(key)
    if (stored === null) return sampleHistory.map((item) => ({ ...item, tags: [...item.tags] }))
    try {
      return JSON.parse(stored) as ClipboardItem[]
    } catch {
      return sampleHistory.map((item) => ({ ...item, tags: [...item.tags] }))
    }
  }

  const items = ref<ClipboardItem[]>(isTauriRuntime() ? [] : loadItems('vertex.clipboardItems'))
  const libraryItems = ref<ClipboardItem[]>(isTauriRuntime() ? [] : loadItems('vertex.libraryItems'))
  const selectedId = ref(items.value[0]?.id ?? '')
  const filter = ref('')
  const autoSave = ref(true)
  const initialized = ref(false)
  const loading = ref(false)
  const lastError = ref('')
  let notesTimer: number | undefined

  const selectedItem = computed(() => items.value.find((item) => item.id === selectedId.value) ?? null)
  const favorites = computed(() => items.value.filter((item) => item.favorite))
  const filteredItems = computed(() => {
    const query = filter.value.trim().toLocaleLowerCase()
    if (!query) return items.value
    return items.value.filter((item) =>
      [item.name, item.format, item.objectType, ...item.tags].some((value) =>
        value.toLocaleLowerCase().includes(query),
      ),
    )
  })

  function select(id: string) {
    selectedId.value = id
  }

  function fromNative(item: NativeClipboardItem): ClipboardItem {
    return { ...item, tags: [] }
  }

  async function initialize() {
    if (initialized.value || loading.value) return
    loading.value = true
    lastError.value = ''
    try {
      if (isTauriRuntime()) {
        items.value = (await nativeGateway.listHistory()).map(fromNative)
        libraryItems.value = (await nativeGateway.listLibrary()).map(fromNative)
      }
      selectedId.value = selectedId.value || items.value[0]?.id || ''
      initialized.value = true
    } catch (error) {
      lastError.value = String(error)
      throw error
    } finally {
      loading.value = false
    }
  }

  async function upsert(
    input: SaveClipboardItemInput,
    options: { select?: boolean } = {},
  ) {
    const shouldSelect = options.select ?? true
    if (!isTauriRuntime()) {
      const now = new Date().toISOString()
      const fallback: ClipboardItem = {
        ...input,
        id: input.id ?? crypto.randomUUID(),
        createdAt: now,
        updatedAt: now,
        lastUsedAt: now,
        favorite: input.favorite ?? false,
        inHistory: input.inHistory ?? true,
        notes: input.notes ?? '',
        tags: [],
      }
      const index = items.value.findIndex((item) => item.id === fallback.id)
      if (index >= 0) items.value[index] = fallback
      else items.value.unshift(fallback)
      if (input.inLibrary) {
        const libraryIndex = libraryItems.value.findIndex((item) => item.id === fallback.id)
        if (libraryIndex >= 0) libraryItems.value[libraryIndex] = { ...fallback, tags: [...fallback.tags] }
        else libraryItems.value.unshift({ ...fallback, tags: [...fallback.tags] })
      }
      if (shouldSelect) selectedId.value = fallback.id
      return fallback
    }
    const saved = fromNative(await nativeGateway.saveItem(input))
    const index = items.value.findIndex((item) => item.id === saved.id)
    if (index >= 0) items.value[index] = saved
    else items.value.unshift(saved)
    const libraryIndex = libraryItems.value.findIndex((item) => item.id === saved.id)
    if (saved.inLibrary) {
      if (libraryIndex >= 0) libraryItems.value[libraryIndex] = { ...saved, tags: [...saved.tags] }
      else libraryItems.value.unshift({ ...saved, tags: [...saved.tags] })
    } else if (libraryIndex >= 0) {
      libraryItems.value.splice(libraryIndex, 1)
    }
    if (shouldSelect) selectedId.value = saved.id
    return saved
  }

  async function toggleFavorite(id = selectedId.value) {
    const item = items.value.find((candidate) => candidate.id === id)
      ?? libraryItems.value.find((candidate) => candidate.id === id)
    if (!item) return
    const favorite = !item.favorite
    for (const candidate of [...items.value, ...libraryItems.value]) {
      if (candidate.id === id) candidate.favorite = favorite
    }
    if (isTauriRuntime()) {
      try {
        await nativeGateway.updateFavorite(id, favorite)
      } catch (error) {
        for (const candidate of [...items.value, ...libraryItems.value]) {
          if (candidate.id === id) candidate.favorite = !favorite
        }
        lastError.value = String(error)
        throw error
      }
    }
  }

  function updateNotes(notes: string) {
    if (selectedItem.value) selectedItem.value.notes = notes
    if (!isTauriRuntime() || !selectedItem.value) return
    window.clearTimeout(notesTimer)
    const id = selectedItem.value.id
    notesTimer = window.setTimeout(() => {
      void nativeGateway.updateNotes(id, notes).catch((error) => { lastError.value = String(error) })
    }, 450)
  }

  async function clearLibrary() {
    if (isTauriRuntime()) await nativeGateway.clearLibrary()
    libraryItems.value = []
  }

  async function clearClipboard() {
    if (isTauriRuntime()) await nativeGateway.clearClipboard()
    items.value = []
    selectedId.value = ''
    filter.value = ''
  }

  async function clearAll() {
    if (isTauriRuntime()) await nativeGateway.clearAll()
    libraryItems.value = []
    items.value = []
    selectedId.value = ''
    filter.value = ''
  }

  watch(items, (value) => {
    if (!isTauriRuntime()) localStorage.setItem('vertex.clipboardItems', JSON.stringify(value))
  }, { deep: true })
  watch(libraryItems, (value) => {
    if (!isTauriRuntime()) localStorage.setItem('vertex.libraryItems', JSON.stringify(value))
  }, { deep: true })

  return {
    items,
    libraryItems,
    selectedId,
    filter,
    autoSave,
    initialized,
    loading,
    lastError,
    selectedItem,
    favorites,
    filteredItems,
    select,
    initialize,
    upsert,
    toggleFavorite,
    updateNotes,
    clearLibrary,
    clearClipboard,
    clearAll,
  }
})
