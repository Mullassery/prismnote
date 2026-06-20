import Editor from '@monaco-editor/react'
import MDPreview from '@uiw/react-markdown-preview'
import { useState } from 'react'
import { Play, Trash2 } from 'lucide-react'
import Output from './Output'
import { useNotebookStore } from '../hooks/useNotebook'

interface CellProps {
  cell: any
  cellIndex: number
}

export default function Cell({ cell, cellIndex }: CellProps) {
  const [isEditing, setIsEditing] = useState(!cell.source.length)
  const { updateCell, deleteCell, executeCell } = useNotebookStore()
  const [isExecuting, setIsExecuting] = useState(false)

  const handleRun = async () => {
    setIsExecuting(true)
    try {
      await executeCell(cellIndex)
    } finally {
      setIsExecuting(false)
    }
  }

  const sourceText = Array.isArray(cell.source) ? cell.source.join('') : cell.source

  return (
    <div className="bg-slate-800 rounded-lg border border-slate-700 overflow-hidden">
      <div className="flex items-center justify-between px-4 py-2 bg-slate-700">
        <div className="flex items-center gap-2">
          <div className="w-5 text-center">
            {isExecuting ? (
              <span className="text-xs text-blue-400">*</span>
            ) : (
              <span className="text-xs text-gray-400">[{cell.execution_count || '-'}]</span>
            )}
          </div>
          <span className="text-xs text-gray-500">{cell.cell_type}</span>
        </div>
        <div className="flex gap-2">
          {cell.cell_type === 'code' && (
            <button
              onClick={handleRun}
              disabled={isExecuting}
              className="p-1 rounded hover:bg-slate-600 disabled:opacity-50"
              title="Shift+Enter"
            >
              <Play size={16} />
            </button>
          )}
          <button
            onClick={() => deleteCell(cellIndex)}
            className="p-1 rounded hover:bg-slate-600 text-red-400"
          >
            <Trash2 size={16} />
          </button>
        </div>
      </div>

      {cell.cell_type === 'code' && (
        <div className="border-t border-slate-700">
          <Editor
            height="200px"
            language="python"
            value={sourceText}
            onChange={(val) => updateCell(cellIndex, { source: val?.split('\n') || [] })}
            theme="vs-dark"
            options={{
              minimap: { enabled: false },
              scrollBeyondLastLine: false,
              lineNumbers: 'on',
              fontSize: 14,
            }}
          />
        </div>
      )}

      {cell.cell_type === 'markdown' && (
        <div className="p-4 border-t border-slate-700">
          {isEditing ? (
            <textarea
              value={sourceText}
              onChange={(e) => updateCell(cellIndex, { source: e.target.value.split('\n') })}
              onBlur={() => setIsEditing(false)}
              className="w-full p-2 bg-slate-700 text-white rounded font-mono text-sm"
              rows={4}
            />
          ) : (
            <div onClick={() => setIsEditing(true)} className="cursor-pointer hover:opacity-80">
              <MDPreview
                source={sourceText}
                style={{ backgroundColor: 'transparent', color: '#e5e7eb' }}
              />
            </div>
          )}
        </div>
      )}

      {cell.outputs.length > 0 && (
        <div className="border-t border-slate-700 bg-slate-900 p-4">
          {cell.outputs.map((output: any, idx: number) => (
            <Output key={idx} output={output} />
          ))}
        </div>
      )}
    </div>
  )
}
