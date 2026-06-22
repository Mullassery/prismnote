import { create } from 'zustand'

// Minimal workspace store backed by the File System Access API. Holds the
// root directory handle the user opened so the Explorer can browse the real
// local filesystem (lazy) and File ▸ Save can write back.

export interface WorkspaceStore {
  rootHandle: FileSystemDirectoryHandle | null
  rootName: string
  /** bump to force the tree to re-read (after create/delete) */
  rev: number
  supported: boolean
  openFolder: () => Promise<void>
  refresh: () => void
  setRoot: (h: FileSystemDirectoryHandle) => void
}

const supported = typeof (window as any).showDirectoryPicker === 'function'

export const useWorkspace = create<WorkspaceStore>((set) => ({
  rootHandle: null,
  rootName: '',
  rev: 0,
  supported,
  setRoot: (h) => set({ rootHandle: h, rootName: h.name, rev: Date.now() }),
  refresh: () => set({ rev: Date.now() }),
  openFolder: async () => {
    if (!supported) {
      alert('Your browser does not support the File System Access API. Use Chrome/Edge.')
      return
    }
    try {
      const handle: FileSystemDirectoryHandle = await (window as any).showDirectoryPicker({ mode: 'readwrite' })
      set({ rootHandle: handle, rootName: handle.name, rev: Date.now() })
    } catch (err: any) {
      if (err?.name === 'AbortError') return // user cancelled — fine
      alert(
        'Could not open the folder picker.\n\n' +
          (err?.message || err) +
          '\n\nThe native folder picker only appears in a normal Chrome window — it cannot show inside an automated/preview browser.'
      )
    }
  },
}))

/** Open a .ipynb from disk and return its parsed JSON + file name. */
export async function openNotebookFile(): Promise<{ name: string; data: any } | null> {
  if (typeof (window as any).showOpenFilePicker !== 'function') {
    alert('Your browser does not support file open. Use Chrome/Edge.')
    return null
  }
  try {
    const [handle] = await (window as any).showOpenFilePicker({
      types: [{ description: 'Notebook', accept: { 'application/json': ['.ipynb', '.json'] } }],
    })
    const file = await handle.getFile()
    const text = await file.text()
    return { name: file.name, data: JSON.parse(text) }
  } catch {
    return null
  }
}

/** Save arbitrary text to disk via the Save dialog (falls back to download). */
export async function saveTextAs(suggestedName: string, text: string) {
  if (typeof (window as any).showSaveFilePicker === 'function') {
    try {
      const handle = await (window as any).showSaveFilePicker({ suggestedName })
      const w = await handle.createWritable()
      await w.write(text)
      await w.close()
      return
    } catch {
      return
    }
  }
  const blob = new Blob([text], { type: 'text/plain' })
  const a = document.createElement('a')
  a.href = URL.createObjectURL(blob)
  a.download = suggestedName
  a.click()
  URL.revokeObjectURL(a.href)
}

/** Save arbitrary JSON to disk via the Save dialog. */
export async function saveJsonAs(suggestedName: string, data: any) {
  const json = JSON.stringify(data, null, 2)
  if (typeof (window as any).showSaveFilePicker === 'function') {
    try {
      const handle = await (window as any).showSaveFilePicker({
        suggestedName,
        types: [{ description: 'Notebook', accept: { 'application/json': ['.ipynb'] } }],
      })
      const w = await handle.createWritable()
      await w.write(json)
      await w.close()
      return
    } catch {
      return
    }
  }
  // fallback: trigger a download
  const blob = new Blob([json], { type: 'application/json' })
  const a = document.createElement('a')
  a.href = URL.createObjectURL(blob)
  a.download = suggestedName
  a.click()
  URL.revokeObjectURL(a.href)
}
