<script setup lang="ts">
import { computed, ref } from 'vue'
import { useQuasar } from 'quasar'
import { useClipboardStore } from '../../stores/clipboard'
import { useEditorStore } from '../../stores/editor'
import { useLocaleStore } from '../../stores/locale'
import { useNavigationStore } from '../../stores/navigation'
import {
  collectionCategories,
  collectionCategoryForItem,
  useCollectionWorkspaceStore,
} from '../../stores/collectionWorkspace'
import { formatXmlForDisplay } from '../../utils/xmlFormat'
import type { ClipboardItem } from '../../types/clipboard'

const clipboard = useClipboardStore()
const editor = useEditorStore()
const collectionWorkspace = useCollectionWorkspaceStore()
const locale = useLocaleStore()
const navigation = useNavigationStore()
const $q = useQuasar()
const search = ref('')
const openingId = ref('')

const category = computed(() =>
  collectionCategories.find((candidate) => candidate.id === collectionWorkspace.selectedCategoryId)
    ?? collectionCategories[0],
)

const projectItems = computed(() => {
  const projectId = collectionWorkspace.selectedProject?.id
  if (!projectId) return []
  return clipboard.libraryItems.filter((item) => collectionWorkspace.projectIdForItem(item.id) === projectId)
})

const visibleItems = computed(() => {
  const query = search.value.trim().toLocaleLowerCase()
  return projectItems.value.filter((item) => {
    const categoryMatches = collectionWorkspace.selectedCategoryId === 'all'
      || collectionCategoryForItem(item) === collectionWorkspace.selectedCategoryId
    const searchMatches = !query
      || [item.name, item.format, item.objectType, ...item.tags]
        .some((value) => value.toLocaleLowerCase().includes(query))
    return categoryMatches && searchMatches
  })
})

function categoryLabel() {
  if (!category.value) return ''
  return locale.language === 'ja' ? category.value.labelJa : category.value.labelEn
}

function objectTypeLabel(item: ClipboardItem) {
  const matched = collectionCategories.find(
    (candidate) => candidate.id === collectionCategoryForItem(item),
  )
  return matched ? (locale.language === 'ja' ? matched.labelJa : matched.labelEn) : item.objectType
}

async function openCollectionItem(source: ClipboardItem) {
  if (openingId.value) return
  openingId.value = source.id
  try {
    const tags = [...source.tags]
    const item = await clipboard.upsert({
      id: source.id,
      name: source.name,
      format: source.format,
      windowsFormat: source.windowsFormat,
      objectType: source.objectType,
      xml: source.xml,
      notes: source.notes,
      favorite: source.favorite,
      inLibrary: true,
      inHistory: true,
    })
    item.tags = tags
    const historyIndex = clipboard.items.findIndex((candidate) => candidate.id === item.id)
    if (historyIndex >= 0) clipboard.items.splice(historyIndex, 1)
    clipboard.items.unshift(item)
    const libraryIndex = clipboard.libraryItems.findIndex((candidate) => candidate.id === item.id)
    if (libraryIndex >= 0) clipboard.libraryItems[libraryIndex] = { ...item, tags }

    const displayXml = formatXmlForDisplay(item.xml)
    editor.content = displayXml
    editor.savedContent = displayXml
    editor.activeTab = 'xml'
    await Promise.all([editor.validate(item.format), editor.buildPreview()])
    navigation.setActive('clipboard')
    $q.notify({
      type: 'positive',
      message: locale.language === 'ja'
        ? `${collectionWorkspace.selectedProject?.name ?? 'Project'}から読み込みました`
        : `Loaded from ${collectionWorkspace.selectedProject?.name ?? 'Project'}`,
    })
  } catch (error) {
    $q.notify({ type: 'negative', message: String(error) })
  } finally {
    openingId.value = ''
  }
}
</script>

<template>
  <main class="collection-browser-workspace">
    <header class="collection-browser-header">
      <div class="collection-browser-heading">
        <div class="collection-browser-icon"><span class="material-icons">folder_open</span></div>
        <div>
          <span>{{ locale.language === 'ja' ? '選択中のコレクションProject' : 'Selected Collection Project' }}</span>
          <h1>{{ collectionWorkspace.selectedProject?.name ?? (locale.language === 'ja' ? 'Project未選択' : 'No project selected') }}</h1>
          <p>
            <span class="material-icons">{{ category?.icon }}</span>
            {{ categoryLabel() }} · {{ visibleItems.length }} {{ locale.language === 'ja' ? '項目' : 'items' }}
          </p>
        </div>
      </div>
      <label class="collection-browser-search">
        <span class="material-icons">search</span>
        <input
          v-model="search"
          type="search"
          :placeholder="locale.language === 'ja' ? 'コレクション内を検索' : 'Search collection'"
        />
      </label>
    </header>

    <section class="collection-browser-body">
      <div class="collection-context-bar">
        <span class="material-icons">account_tree</span>
        <strong>{{ collectionWorkspace.selectedProject?.name }}</strong>
        <span class="material-icons separator">chevron_right</span>
        <span>{{ categoryLabel() }}</span>
        <em>{{ visibleItems.length }}</em>
      </div>

      <div v-if="visibleItems.length" class="collection-item-grid">
        <article
          v-for="item in visibleItems"
          :key="item.id"
          class="collection-item-card"
          :class="{ selected: clipboard.selectedId === item.id }"
          role="button"
          tabindex="0"
          @click="openCollectionItem(item)"
          @keydown.enter.prevent="openCollectionItem(item)"
          @keydown.space.prevent="openCollectionItem(item)"
        >
          <header>
            <span class="collection-format-pill">{{ item.format }}</span>
            <button
              type="button"
              :aria-label="item.favorite ? locale.t('removeFavorite') : locale.t('addFavorite')"
              @click.stop="clipboard.toggleFavorite(item.id)"
            >
              <span class="material-icons" :class="{ active: item.favorite }">{{ item.favorite ? 'star' : 'star_border' }}</span>
            </button>
          </header>
          <strong>{{ item.name }}</strong>
          <small>{{ objectTypeLabel(item) }} · {{ item.windowsFormat }}</small>
          <div v-if="item.tags.length" class="collection-card-tags">
            <span v-for="tag in item.tags.slice(0, 3)" :key="tag">{{ tag }}</span>
          </div>
          <footer>
            <span class="material-icons">open_in_new</span>
            {{ locale.language === 'ja' ? 'エディターで開く' : 'Open in editor' }}
          </footer>
        </article>
      </div>

      <div v-else class="collection-browser-empty">
        <span class="material-icons">folder_off</span>
        <strong>{{ locale.language === 'ja' ? 'このフォルダには項目がありません' : 'This folder is empty' }}</strong>
        <p>{{ locale.language === 'ja' ? '履歴カードまたは右側の操作パネルからライブラリへ保存してください。' : 'Save an item to the library from a history card or the actions panel.' }}</p>
      </div>
    </section>
  </main>
</template>
