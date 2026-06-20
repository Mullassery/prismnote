import { useState, useEffect } from 'react'
import { useNotebookStore } from '../hooks/useNotebook'
import Cell from './Cell'
import Toolbar from './Toolbar'
import AIPanel from './AIPanel'
import LibrarySuggester from './LibrarySuggester'
import { Plus } from 'lucide-react'

export default function Notebook() {
  const {
    currentNotebook,
    addCell,
    updateCell,
    librarySuggestions,
    suggestionsIntent,
    suggestionsSummary,
    suggestionsLoading,
    suggestLibraries,
    ignoreLibrary,
  } = useNotebookStore()
  const [selectedCellIndex, setSelectedCellIndex] = useState<number | null>(null)
  const [rightPanelMode, setRightPanelMode] = useState<'ai' | 'libraries'>('ai')

  useEffect(() => {
    if (currentNotebook) {
      suggestLibraries()
    }
  }, [currentNotebook?.id])

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

      {/* Right Panel - AI or Libraries */}
      <div className="w-80 border-l border-slate-700 flex flex-col overflow-hidden">
        {/* Panel tabs */}
        <div className="flex gap-0 border-b border-slate-700">
          <button
            onClick={() => setRightPanelMode('ai')}
            className={`flex-1 px-3 py-2 text-xs font-medium transition ${
              rightPanelMode === 'ai'
                ? 'border-b-2 border-blue-400 text-blue-400'
                : 'text-gray-400 hover:text-white border-b-2 border-transparent'
            }`}
          >
            AI
          </button>
          <button
            onClick={() => setRightPanelMode('libraries')}
            className={`flex-1 px-3 py-2 text-xs font-medium transition ${
              rightPanelMode === 'libraries'
                ? 'border-b-2 border-blue-400 text-blue-400'
                : 'text-gray-400 hover:text-white border-b-2 border-transparent'
            }`}
          >
            Libraries
          </button>
        </div>

        {/* Panel content */}
        <div className="flex-1 overflow-hidden">
          {rightPanelMode === 'ai' && selectedCell && selectedCell.cell_type === 'code' ? (
            <AIPanel
              cellCode={cellCode}
              cellError={selectedCell.outputs?.find((o: any) => o.output_type === 'error')?.text?.[0]}
              onInsertCode={handleInsertAICode}
            />
          ) : rightPanelMode === 'libraries' ? (
            <LibrarySuggester
              suggestions={librarySuggestions}
              onInstall={(name, version) => {
                console.log(`Install ${name}@${version}`)
              }}
              onIgnore={ignoreLibrary}
              isLoading={suggestionsLoading}
              detectedIntent={suggestionsIntent}
              contextSummary={suggestionsSummary}
            />
          ) : (
            <div className="p-4 text-gray-500 text-sm">Select a code cell to use AI assistance</div>
          )}
        </div>
      </div>
    </div>
  )
}
