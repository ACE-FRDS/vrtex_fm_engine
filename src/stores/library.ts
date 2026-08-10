import { ref, watch } from 'vue'
import { defineStore } from 'pinia'
import { sampleCollections } from '../data/sample'

export const useLibraryStore = defineStore('library', () => {
  const storedCollections = localStorage.getItem('vertex.collections')
  let initialCollections = sampleCollections
  if (storedCollections !== null) {
    try {
      initialCollections = JSON.parse(storedCollections)
    } catch {
      initialCollections = sampleCollections
    }
  }
  const collections = ref(initialCollections)
  const selectedCollectionId = ref<string | null>(null)

  function clearCollections() {
    collections.value = []
    selectedCollectionId.value = null
  }

  watch(collections, (value) => localStorage.setItem('vertex.collections', JSON.stringify(value)), { deep: true })

  return { collections, selectedCollectionId, clearCollections }
})
