import { invoke } from '@tauri-apps/api/core'
import type { ClipboardFormat, ObjectType, ValidationResult } from '../types/clipboard'

export function isTauriRuntime() {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
}

export interface NativeClipboardItem {
  id: string
  name: string
  format: ClipboardFormat
  windowsFormat: string
  objectType: ObjectType
  xml: string
  checksum: string
  filemakerVersion: string | null
  notes: string
  favorite: boolean
  inLibrary: boolean
  inHistory: boolean
  createdAt: string
  updatedAt: string
  lastUsedAt: string
}

export interface SaveClipboardItemInput {
  id?: string
  name: string
  format: ClipboardFormat
  windowsFormat: string
  objectType: ObjectType
  xml: string
  filemakerVersion?: string | null
  notes?: string
  favorite?: boolean
  inLibrary?: boolean
  inHistory?: boolean
}

export interface DetectionResult {
  format: ClipboardFormat
  objectType: ObjectType
  confidence: number
  evidence: string[]
}

interface NativeValidationIssue extends ValidationResult {
  code: string
}

export interface ValidationReport {
  valid: boolean
  detectedFormat: ClipboardFormat
  issues: NativeValidationIssue[]
}

export interface ScriptPreviewStep {
  index: number
  name: string
  enabled: boolean
}

export interface ScriptPreview {
  name: string
  steps: ScriptPreviewStep[]
}

export interface FileMakerStatus {
  detected: boolean
  version: string | null
  displayName: string
}

export const nativeGateway = {
  listHistory(limit = 500) {
    return invoke<NativeClipboardItem[]>('list_clipboard_history', { limit })
  },
  listLibrary(limit = 500) {
    return invoke<NativeClipboardItem[]>('list_library_items', { limit })
  },
  saveItem(item: SaveClipboardItemInput) {
    return invoke<NativeClipboardItem>('save_clipboard_item', { item })
  },
  deleteItem(id: string) {
    return invoke<boolean>('delete_clipboard_item', { id })
  },
  updateFavorite(id: string, favorite: boolean) {
    return invoke<boolean>('update_clipboard_favorite', { id, favorite })
  },
  updateNotes(id: string, notes: string) {
    return invoke<boolean>('update_clipboard_notes', { id, notes })
  },
  clearClipboard() {
    return invoke<number>('clear_clipboard_history')
  },
  clearLibrary() {
    return invoke<number>('clear_library_data')
  },
  clearAll() {
    return invoke<number>('clear_all_data')
  },
  detectFormat(xml: string) {
    return invoke<DetectionResult>('detect_xml_format', { xml })
  },
  validateXml(xml: string, format?: string) {
    return invoke<ValidationReport>('validate_filemaker_xml', { xml, format: format ?? null })
  },
  previewXml(xml: string) {
    return invoke<ScriptPreview>('preview_filemaker_xml', { xml })
  },
  detectFileMaker() {
    return invoke<FileMakerStatus>('detect_filemaker')
  },
}
