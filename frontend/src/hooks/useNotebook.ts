import { create } from 'zustand'
import axios from 'axios'

interface Cell {
  id: string
  cell_type: 'code' | 'markdown'
  source: string[]
  outputs: any[]
  execution_count: number | null
  metadata: Record<string, any>
}

interface Notebook {
  id: string
  name: string
  cells: Cell[]
  metadata: Record<string, any>
}

interface LibrarySuggestion {
  name: string
  version: string
  description: string
  reasoning: string
  installed_version?: string
  is_update: boolean
  category: 'data' | 'viz' | 'ml' | 'web' | 'utility'
  confidence: number
}

interface SuggestionsResponse {
  suggestions: LibrarySuggestion[]
  detected_intent: string
  context_summary: string
}

interface NotebookStore {
  notebooks: Notebook[]
  currentNotebookId: string | null
  currentNotebook: Notebook | null
  librarySuggestions: LibrarySuggestion[]
  suggestionsIntent: string
  suggestionsSummary: string
  suggestionsLoading: boolean
  createNotebook: (name: string) => void
  deleteNotebook: (id: string) => void
  setCurrentNotebook: (id: string) => void
  addCell: (type: 'code' | 'markdown', index?: number) => void
  updateCell: (index: number, updates: Partial<Cell>) => void
  deleteCell: (index: number) => void
  executeCell: (index: number) => Promise<void>
  suggestLibraries: () => Promise<void>
  ignoreLibrary: (libraryName: string) => Promise<void>
  saveNotebook: () => void
}

const API_BASE = '/api'

export const useNotebookStore = create<NotebookStore>((set, get) => ({
  notebooks: [],
  currentNotebookId: null,
  currentNotebook: null,
  librarySuggestions: [],
  suggestionsIntent: '',
  suggestionsSummary: '',
  suggestionsLoading: false,

  createNotebook: async (name: string) => {
    try {
      const res = await axios.post(`${API_BASE}/notebooks`, { name })
      const newNotebook = res.data
      set((state) => ({
        notebooks: [...state.notebooks, newNotebook],
        currentNotebookId: newNotebook.id,
        currentNotebook: newNotebook,
      }))
    } catch (err) {
      console.error('Failed to create notebook:', err)
    }
  },

  deleteNotebook: async (id: string) => {
    try {
      await axios.delete(`${API_BASE}/notebooks/${id}`)
      set((state) => {
        const notebooks = state.notebooks.filter((n) => n.id !== id)
        const newCurrent =
          state.currentNotebookId === id ? notebooks[0]?.id || null : state.currentNotebookId
        return {
          notebooks,
          currentNotebookId: newCurrent,
          currentNotebook: notebooks.find((n) => n.id === newCurrent) || null,
        }
      })
    } catch (err) {
      console.error('Failed to delete notebook:', err)
    }
  },

  setCurrentNotebook: async (id: string) => {
    try {
      const res = await axios.get(`${API_BASE}/notebooks/${id}`)
      set({
        currentNotebookId: id,
        currentNotebook: res.data,
      })
    } catch (err) {
      console.error('Failed to load notebook:', err)
    }
  },

  addCell: (type: 'code' | 'markdown', index?: number) => {
    set((state) => {
      if (!state.currentNotebook) return state

      const newCell: Cell = {
        id: Math.random().toString(36),
        cell_type: type,
        source: [],
        outputs: [],
        execution_count: null,
        metadata: {},
      }

      const cells = [...state.currentNotebook.cells]
      // index = insert position; default (undefined) appends to the end.
      const at = index == null ? cells.length : Math.max(0, Math.min(index, cells.length))
      cells.splice(at, 0, newCell)

      return {
        currentNotebook: { ...state.currentNotebook, cells },
      }
    })
  },

  updateCell: (index: number, updates: Partial<Cell>) => {
    set((state) => {
      if (!state.currentNotebook) return state

      const cells = [...state.currentNotebook.cells]
      cells[index] = { ...cells[index], ...updates }

      const updated = {
        currentNotebook: {
          ...state.currentNotebook,
          cells,
        },
      }

      // Auto-save after a short delay
      setTimeout(() => {
        get().saveNotebook()
      }, 1000)

      return updated
    })
  },

  deleteCell: (index: number) => {
    set((state) => {
      if (!state.currentNotebook) return state

      return {
        currentNotebook: {
          ...state.currentNotebook,
          cells: state.currentNotebook.cells.filter((_, i) => i !== index),
        },
      }
    })
  },

  executeCell: async (index: number) => {
    const state = get()
    if (!state.currentNotebook) return

    const cell = state.currentNotebook.cells[index]
    if (!cell) return

    try {
      const res = await axios.post(`${API_BASE}/notebooks/${state.currentNotebook.id}/execute`, {
        cell_id: cell.id,
        // send the code directly so execution doesn't depend on the on-disk file
        code: Array.isArray(cell.source) ? cell.source.join('') : cell.source,
      })

      const { execution_count, outputs } = res.data
      set((s) => {
        if (!s.currentNotebook) return s

        const cells = [...s.currentNotebook.cells]
        cells[index] = {
          ...cells[index],
          execution_count,
          outputs,
        }

        return {
          currentNotebook: {
            ...s.currentNotebook,
            cells,
          },
        }
      })

      // Auto-save after execution
      setTimeout(() => {
        get().saveNotebook()
      }, 500)

      // Suggest libraries after execution (debounced)
      setTimeout(() => {
        get().suggestLibraries()
      }, 1000)
    } catch (err: any) {
      console.error('Failed to execute cell:', err)

      // Show error in output
      set((s) => {
        if (!s.currentNotebook) return s

        const cells = [...s.currentNotebook.cells]
        cells[index] = {
          ...cells[index],
          outputs: [
            {
              output_type: 'error',
              text: [err.response?.data?.message || err.message || 'Execution failed'],
              metadata: null,
              data: null,
            },
          ],
        }

        return {
          currentNotebook: {
            ...s.currentNotebook,
            cells,
          },
        }
      })
    }
  },

  suggestLibraries: async () => {
    const state = get()
    if (!state.currentNotebook) return

    set({ suggestionsLoading: true })

    try {
      const notebookCode = state.currentNotebook.cells
        .filter((c) => c.cell_type === 'code')
        .map((c) => (Array.isArray(c.source) ? c.source.join('') : c.source))
        .join('\n\n')

      const res = await axios.post(`${API_BASE}/notebooks/${state.currentNotebook.id}/suggest-libraries`, {
        notebook_code: notebookCode,
        installed_packages: [],
        ignored_libraries: [],
      } as any)

      const data: SuggestionsResponse = res.data
      set({
        librarySuggestions: data.suggestions,
        suggestionsIntent: data.detected_intent,
        suggestionsSummary: data.context_summary,
        suggestionsLoading: false,
      })
    } catch (err) {
      console.error('Failed to suggest libraries:', err)
      set({ suggestionsLoading: false })
    }
  },

  ignoreLibrary: async (libraryName: string) => {
    const state = get()
    if (!state.currentNotebook) return

    try {
      await axios.post(`${API_BASE}/notebooks/${state.currentNotebook.id}/libraries/ignore`, {
        library_name: libraryName,
        reason: 'User ignored',
      })

      // Remove from suggestions
      set({
        librarySuggestions: state.librarySuggestions.filter((s) => s.name !== libraryName),
      })
    } catch (err) {
      console.error('Failed to ignore library:', err)
    }
  },

  saveNotebook: async () => {
    const state = get()
    if (!state.currentNotebook) return

    try {
      await axios.put(`${API_BASE}/notebooks/${state.currentNotebook.id}`, {
        notebook: state.currentNotebook,
      })
      console.log('Notebook saved successfully')
    } catch (err) {
      console.error('Failed to save notebook:', err)
    }
  },
}))
