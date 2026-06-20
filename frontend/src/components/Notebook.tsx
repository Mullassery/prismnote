import { useNotebookStore } from '../hooks/useNotebook'
import Cell from './Cell'
import Toolbar from './Toolbar'
import { Plus } from 'lucide-react'

export default function Notebook() {
  const { currentNotebook, addCell } = useNotebookStore()

  if (!currentNotebook) {
    return <div className="p-4">No notebook selected</div>
  }

  return (
    <div className="h-full flex flex-col bg-slate-900">
      <Toolbar />
      <div className="flex-1 overflow-y-auto p-4">
        <div className="max-w-4xl mx-auto space-y-3">
          {currentNotebook.cells.map((cell, idx) => (
            <Cell key={cell.id} cell={cell} cellIndex={idx} />
          ))}

          <button
            onClick={() => addCell('code')}
            className="flex items-center gap-2 p-3 rounded hover:bg-slate-800 text-gray-400 hover:text-white transition"
          >
            <Plus size={18} />
            Add Code Cell
          </button>
        </div>
      </div>
    </div>
  )
}
