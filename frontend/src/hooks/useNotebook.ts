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

interface NotebookStore {
  notebooks: Notebook[]
  currentNotebookId: string | null
  currentNotebook: Notebook | null
  createNotebook: (name: string) => void
  deleteNotebook: (id: string) => void
  setCurrentNotebook: (id: string) => void
  addCell: (type: 'code' | 'markdown') => void
  updateCell: (index: number, updates: Partial<Cell>) => void
  deleteCell: (index: number) => void
  executeCell: (index: number) => Promise<void>
  saveNotebook: () => void
}

const API_BASE = 'http://localhost:8000/api'

export const useNotebookStore = create<NotebookStore>((set, get) => ({
  notebooks: [],
  currentNotebookId: null,
  currentNotebook: null,

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

  addCell: (type: 'code' | 'markdown') => {
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

      return {
        currentNotebook: {
          ...state.currentNotebook,
          cells: [...state.currentNotebook.cells, newCell],
        },
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
