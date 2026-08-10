<script setup lang="ts">
import { computed, nextTick, ref } from 'vue'
import { useQuasar } from 'quasar'
import { useClipboardStore } from '../../stores/clipboard'
import { useLibraryStore } from '../../stores/library'
import { useLocaleStore } from '../../stores/locale'
import type { ClipboardItem } from '../../types/clipboard'

type SidebarView = 'history' | 'collections' | 'favorites'

const clipboard = useClipboardStore()
const library = useLibraryStore()
const locale = useLocaleStore()
const $q = useQuasar()
const activeView = ref<SidebarView>('history')
const editingTitleId = ref('')
const titleDraft = ref('')

const displayedItems = computed(() =>
  activeView.value === 'favorites' ? clipboard.favorites : clipboard.filteredItems,
)

const groups = computed(() => {
  const grouped = new Map<string, ClipboardItem[]>()
  for (const item of displayedItems.value) {
    const created = new Date(item.createdAt)
    const today = new Date()
    const dayDiff = Math.floor(
      (new Date(today.getFullYear(), today.getMonth(), today.getDate()).getTime() -
        new Date(created.getFullYear(), created.getMonth(), created.getDate()).getTime()) /
        86_400_000,
    )
    const label = dayDiff === 0 ? locale.t('today') : dayDiff === 1 ? locale.t('yesterday') : locale.t('earlier')
    if (!grouped.has(label)) grouped.set(label, [])
    grouped.get(label)?.push(item)
  }
  return [...grouped.entries()]
})

function timeLabel(iso: string) {
  return new Intl.DateTimeFormat(locale.language === 'ja' ? 'ja-JP' : 'en-US', {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    hour12: false,
  }).format(new Date(iso))
}

function isMultipleScripts(item: ClipboardItem) {
  if (item.format !== 'XMSC') return false
  const document = new DOMParser().parseFromString(item.xml, 'application/xml')
  if (document.querySelector('parsererror')) return false
  return document.querySelectorAll('Script').length > 1
}

function objectTypeLabel(item: ClipboardItem) {
  return isMultipleScripts(item)
    ? locale.t('multipleScripts')
    : item.objectType
}

async function startTitleEdit(item: ClipboardItem) {
  editingTitleId.value = item.id
  titleDraft.value = item.name
  await nextTick()
  const input = document.querySelector<HTMLInputElement>(
    `.card-title-editor[data-item-id="${CSS.escape(item.id)}"]`,
  )
  input?.focus()
  input?.select()
}

function cancelTitleEdit() {
  editingTitleId.value = ''
  titleDraft.value = ''
}

async function saveTitle(item: ClipboardItem) {
  if (editingTitleId.value !== item.id) return
  const name = titleDraft.value.trim()
  cancelTitleEdit()
  if (!name) {
    $q.notify({ type: 'warning', message: locale.t('cardTitleRequired') })
    return
  }
  if (name === item.name) return
  try {
    await clipboard.upsert({
      id: item.id,
      name,
      format: item.format,
      windowsFormat: item.windowsFormat,
      objectType: item.objectType,
      xml: item.xml,
      notes: item.notes,
      favorite: item.favorite,
      inLibrary: item.inLibrary,
      inHistory: item.inHistory,
    }, { select: false })
    $q.notify({ type: 'positive', message: locale.t('cardTitleUpdated') })
  } catch (error) {
    $q.notify({ type: 'negative', message: String(error) })
  }
}
</script>

<template>
  <aside class="sidebar-panel">
    <div class="side-tabs">
      <button
        v-for="tab in ([
          ['history', 'history', locale.t('history')],
          ['collections', 'account_tree', locale.t('navCollections')],
          ['favorites', 'star_border', locale.t('favorites')],
        ] as const)"
        :key="tab[0]"
        :class="{ active: activeView === tab[0] }"
        :title="tab[2]"
        type="button"
        @click="activeView = tab[0]"
      >
        <span class="material-icons">{{ tab[1] }}</span>
        <small>{{ tab[2] }}</small>
      </button>
    </div>

    <section class="side-content">
      <div class="panel-heading">
        <div>
          <span class="eyebrow">{{ locale.t('navClipboard') }}</span>
          <h2>{{ activeView === 'collections' ? locale.t('navCollections') : activeView === 'favorites' ? locale.t('favorites') : locale.t('history') }}</h2>
        </div>
        <q-btn flat dense round size="sm" icon="more_horiz" />
      </div>

      <label v-if="activeView !== 'collections'" class="search-box">
        <span class="material-icons">search</span>
        <input v-model="clipboard.filter" type="search" :placeholder="locale.t('filterItems')" />
        <kbd>⌘K</kbd>
      </label>

      <div v-if="activeView === 'collections'" class="collection-list">
        <div v-for="collection in library.collections" :key="collection.id" class="collection-group">
          <button type="button" class="collection-row root">
            <span class="material-icons">keyboard_arrow_down</span>
            <span class="material-icons folder">folder_open</span>
            <span>{{ collection.name }}</span>
            <em>{{ collection.count }}</em>
          </button>
          <button
            v-for="child in collection.children"
            :key="child.id"
            type="button"
            class="collection-row child"
          >
            <span class="tree-line" />
            <span class="material-icons folder">folder</span>
            <span>{{ child.name }}</span>
            <em>{{ child.count }}</em>
          </button>
        </div>
      </div>

      <div v-else class="history-scroll">
        <section v-for="[label, items] in groups" :key="label" class="history-group">
          <div class="group-label"><span>{{ label }}</span><i /></div>
          <div
            v-for="item in items"
            :key="item.id"
            class="history-card"
            :class="{ selected: clipboard.selectedId === item.id }"
            role="button"
            tabindex="0"
            @click="clipboard.select(item.id)"
            @keydown.enter="clipboard.select(item.id)"
            @keydown.space.prevent="clipboard.select(item.id)"
          >
            <div class="card-accent" />
            <div class="history-main">
              <div class="history-title-row">
                <input
                  v-if="editingTitleId === item.id"
                  v-model="titleDraft"
                  class="card-title-editor"
                  :data-item-id="item.id"
                  :aria-label="locale.t('editCardTitle')"
                  maxlength="120"
                  @click.stop
                  @blur="saveTitle(item)"
                  @keydown.enter.stop.prevent="saveTitle(item)"
                  @keydown.escape.stop.prevent="cancelTitleEdit"
                />
                <strong v-else>{{ item.name }}</strong>
                <div class="history-title-actions">
                  <button
                    v-if="isMultipleScripts(item) && editingTitleId !== item.id"
                    class="title-edit-toggle"
                    type="button"
                    :aria-label="locale.t('editCardTitle')"
                    :title="locale.t('editCardTitle')"
                    @click.stop="startTitleEdit(item)"
                  >
                    <span class="material-icons">edit</span>
                  </button>
                  <button
                    class="favorite-toggle"
                    :class="{ active: item.favorite }"
                    type="button"
                    :aria-label="item.favorite ? locale.t('removeFavorite') : locale.t('addFavorite')"
                    :title="item.favorite ? locale.t('removeFavorite') : locale.t('addFavorite')"
                    @click.stop="clipboard.toggleFavorite(item.id)"
                  >
                    <span class="material-icons">{{ item.favorite ? 'star' : 'star_border' }}</span>
                  </button>
                </div>
              </div>
              <div class="history-meta">
                <span class="format-pill">{{ item.format }}</span>
                <span class="object-type">{{ objectTypeLabel(item) }}</span>
                <time>{{ timeLabel(item.createdAt) }}</time>
              </div>
              <div v-if="item.tags.length" class="mini-tags">
                <span v-for="tag in item.tags.slice(0, 3)" :key="tag">{{ tag }}</span>
              </div>
            </div>
          </div>
        </section>
        <div v-if="groups.length === 0" class="empty-state">
          <span class="material-icons">inbox</span>
          <p>{{ locale.t('noItems') }}</p>
        </div>
      </div>
    </section>
  </aside>
</template>
