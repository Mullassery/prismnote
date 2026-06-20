import { useState } from 'react'
import { Plus, FileText, Trash2, Sun, Moon } from 'lucide-react'
import { useNotebookStore } from '../hooks/useNotebook'

interface SidebarProps {
  onToggleTheme?: () => void
  currentTheme?: 'light' | 'dark'
}

export default function Sidebar({ onToggleTheme, currentTheme = 'dark' }: SidebarProps) {
  const [newName, setNewName] = useState('')
  const [isCreating, setIsCreating] = useState(false)
  const { notebooks, currentNotebookId, createNotebook, deleteNotebook, setCurrentNotebook } =
    useNotebookStore()

  const handleCreate = () => {
    if (newName.trim()) {
      createNotebook(newName)
      setNewName('')
      setIsCreating(false)
    }
  }

  return (
    <aside className="w-64 bg-slate-950 border-r border-slate-800 flex flex-col">
      <div className="p-4 border-b border-slate-800">
        <h1 className="text-lg font-bold text-white mb-4">PrismNote</h1>

        {isCreating ? (
          <div className="space-y-2">
            <input
              autoFocus
              value={newName}
              onChange={(e) => setNewName(e.target.value)}
              placeholder="Notebook name"
              className="w-full px-3 py-2 bg-slate-800 border border-slate-700 rounded text-sm text-white placeholder-gray-500"
              onKeyDown={(e) => {
                if (e.key === 'Enter') handleCreate()
                if (e.key === 'Escape') setIsCreating(false)
              }}
            />
            <div className="flex gap-2">
              <button
                onClick={handleCreate}
                className="flex-1 px-2 py-1 bg-blue-600 hover:bg-blue-700 rounded text-xs text-white"
              >
                Create
              </button>
              <button
                onClick={() => setIsCreating(false)}
                className="flex-1 px-2 py-1 bg-slate-700 hover:bg-slate-600 rounded text-xs text-white"
              >
                Cancel
              </button>
            </div>
          </div>
        ) : (
          <button
            onClick={() => setIsCreating(true)}
            className="w-full flex items-center justify-center gap-2 px-3 py-2 bg-blue-600 hover:bg-blue-700 rounded text-sm text-white font-medium"
          >
            <Plus size={16} />
            New Notebook
          </button>
        )}
      </div>

      <div className="flex-1 overflow-y-auto p-3 space-y-2">
        {notebooks.map((nb) => (
          <div
            key={nb.id}
            onClick={() => setCurrentNotebook(nb.id)}
            className={`flex items-center justify-between p-3 rounded cursor-pointer transition ${
              currentNotebookId === nb.id
                ? 'bg-slate-700 text-white'
                : 'text-gray-300 hover:bg-slate-800'
            }`}
          >
            <div className="flex items-center gap-2 flex-1 min-w-0">
              <FileText size={16} className="flex-shrink-0" />
              <span className="text-sm truncate">{nb.name}</span>
            </div>
            <button
              onClick={(e) => {
                e.stopPropagation()
                deleteNotebook(nb.id)
              }}
              className="p-1 hover:bg-red-600 rounded transition text-gray-400 hover:text-white"
            >
              <Trash2 size={14} />
            </button>
          </div>
        ))}
      </div>

      <div className="p-3 border-t border-slate-800 space-y-3">
        <div className="text-xs text-gray-500">
          <p>Shift+Enter to run</p>
          <p>Ctrl+Enter for output only</p>
        </div>
        {onToggleTheme && (
          <button
            onClick={onToggleTheme}
            className="w-full flex items-center justify-center gap-2 px-3 py-2 bg-slate-800 hover:bg-slate-700 rounded text-xs text-gray-300 hover:text-white transition"
            aria-label={`Switch to ${currentTheme === 'light' ? 'dark' : 'light'} mode`}
          >
            {currentTheme === 'light' ? (
              <>
                <Moon size={14} />
                Dark Mode
              </>
            ) : (
              <>
                <Sun size={14} />
                Light Mode
              </>
            )}
          </button>
        )}
      </div>
    </aside>
  )
}
