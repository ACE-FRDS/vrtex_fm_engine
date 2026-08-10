import { invoke } from '@tauri-apps/api/core'

export interface FileMakerClipboardPayload {
  format: string
  windowsFormat: string
  xml: string
  rawSize: number
}

export interface ClipboardGateway {
  get(): Promise<FileMakerClipboardPayload>
  set(format: string, xml: string): Promise<void>
}

class TauriClipboardGateway implements ClipboardGateway {
  get() {
    return invoke<FileMakerClipboardPayload>('get_filemaker_clipboard')
  }

  set(format: string, xml: string) {
    return invoke<void>('set_filemaker_clipboard', { format, xml })
  }
}

export const clipboardGateway: ClipboardGateway = new TauriClipboardGateway()
