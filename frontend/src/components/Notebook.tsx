import { useState } from 'react'
import { useNotebookStore } from '../hooks/useNotebook'
import Cell from './Cell'
import Toolbar from './Toolbar'
import AIPanel from './AIPanel'
import { Plus } from 'lucide-react'

export default function Notebook() {
  const { currentNotebook, addCell, updateCell } = useNotebookStore()
  const [selectedCellIndex, setSelectedCellIndex] = useState<number | null>(null)

  if (!currentNotebook) {
    return <div className="p-4">No notebook selected</div>
  }

  const selectedCell = selectedCellIndex !== null ? currentNotebook.cells[selectedCellIndex] : null
  const cellCode = selectedCell ? (Array.isArray(selectedCell.source) ? selectedCell.source.join('') : selectedCell.source) : ''

  const handleInsertAICode = (code: string) => {
    if (selectedCellIndex !== null) {
      updateCell(selectedCellIndex, { source: code.split('\n') })
      setSelectedCellIndex(null)
    }
  }

  return (
    <div className="h-full flex bg-slate-900">
      {/* Main notebook area */}
      <div className="flex-1 flex flex-col overflow-hidden">
        <Toolbar />
        <div className="flex-1 overflow-y-auto p-4">
          <div className="max-w-4xl mx-auto space-y-3">
            {currentNotebook.cells.map((cell, idx) => (
              <div
                key={cell.id}
                onClick={() => setSelectedCellIndex(idx)}
                className={`cursor-pointer transition ${selectedCellIndex === idx ? 'ring-2 ring-blue-500 rounded-lg' : ''}`}
              >
                <Cell cell={cell} cellIndex={idx} />
              </div>
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

      {/* AI Panel */}
      {selectedCell && selectedCell.cell_type === 'code' && (
        <AIPanel
          cellCode={cellCode}
          cellError={selectedCell.outputs?.find((o: any) => o.output_type === 'error')?.text?.[0]}
          onInsertCode={handleInsertAICode}
        />
      )}
    </div>
  )
}
