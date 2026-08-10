export type ClipboardFormat =
  | 'XMSC'
  | 'XMSS'
  | 'XMTB'
  | 'XMFD'
  | 'XML2'
  | 'XMFN'
  | 'XMTH'
  | 'UNKNOWN'
  | (string & {})

export type ObjectType =
  | 'Script'
  | 'Step'
  | 'Table'
  | 'Field'
  | 'Layout'
  | 'Custom Function'
  | 'Theme'
  | 'Unknown'

export interface ClipboardItem {
  id: string
  name: string
  format: ClipboardFormat
  windowsFormat: string
  objectType: ObjectType
  xml: string
  createdAt: string
  updatedAt: string
  lastUsedAt: string
  favorite: boolean
  tags: string[]
  notes: string
  inLibrary?: boolean
  inHistory?: boolean
}

export interface ValidationResult {
  level: 'success' | 'warning' | 'error'
  message: string
}

export interface InspectorDetails {
  windowsFormat: string
  internalFormat: string
  objectType: string
  objects: string
  fileMakerVersion: string
  size: number
  encoding: string
  headerBytes: number
}
