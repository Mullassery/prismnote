import { useState } from 'react'
import { Eye, EyeOff, RefreshCw, Trash2 } from 'lucide-react'

interface Variable {
  name: string
  type: string
  value: string
  size?: string
}

interface VariableInspectorProps {
  variables: Variable[]
  onRefresh: () => void
  onClear: () => void
  isLoading?: boolean
}

export default function VariableInspector({
  variables,
  onRefresh,
  onClear,
  isLoading = false,
}: VariableInspectorProps) {
  const [expanded, setExpanded] = useState(true)
  const [filter, setFilter] = useState('')

  const filteredVars = variables.filter((v) =>
    v.name.toLowerCase().includes(filter.toLowerCase())
  )

  return (
    <div className="bg-slate-800 rounded border border-slate-700 h-full flex flex-col">
      {/* Header */}
      <div className="p-3 border-b border-slate-700 flex items-center justify-between">
        <button
          onClick={() => setExpanded(!expanded)}
          className="flex items-center gap-2 flex-1 font-semibold text-white hover:opacity-80 transition"
        >
          {expanded ? <Eye size={16} /> : <EyeOff size={16} />}
          Variables ({filteredVars.length})
        </button>
        <div className="flex gap-1">
          <button
            onClick={onRefresh}
            disabled={isLoading}
            className="p-1 hover:bg-slate-700 rounded transition disabled:opacity-50"
            title="Refresh variables"
          >
            <RefreshCw size={14} className={isLoading ? 'animate-spin' : ''} />
          </button>
          <button
            onClick={onClear}
            className="p-1 hover:bg-red-900 rounded transition"
            title="Clear all variables"
          >
            <Trash2 size={14} className="text-red-400" />
          </button>
        </div>
      </div>

      {!expanded && (
        <div className="text-xs text-gray-500 p-2 text-center">Click to expand</div>
      )}

      {expanded && (
        <>
          {/* Search */}
          <div className="p-2 border-b border-slate-700">
            <input
              type="text"
              value={filter}
              onChange={(e) => setFilter(e.target.value)}
              placeholder="Search..."
              className="w-full px-2 py-1 bg-slate-700 border border-slate-600 rounded text-xs text-white placeholder-gray-500"
            />
          </div>

          {/* Variables List */}
          <div className="flex-1 overflow-y-auto p-2 space-y-1">
            {filteredVars.length === 0 ? (
              <p className="text-xs text-gray-500 text-center py-4">
                No variables yet. Run a cell to create them.
              </p>
            ) : (
              filteredVars.map((v) => (
                <div
                  key={v.name}
                  className="p-2 bg-slate-700 rounded text-xs hover:bg-slate-600 transition"
                >
                  <div className="flex items-start justify-between gap-2">
                    <div className="flex-1 min-w-0">
                      <p className="font-mono text-blue-300 truncate">{v.name}</p>
                      <p className="text-gray-400 text-xs mt-0.5">{v.type}</p>
                    </div>
                    {v.size && (
                      <p className="text-gray-500 text-xs whitespace-nowrap">{v.size}</p>
                    )}
                  </div>
                  <p className="text-gray-300 mt-1 break-words max-h-16 overflow-hidden">
                    {v.value.length > 100 ? `${v.value.slice(0, 100)}...` : v.value}
                  </p>
                </div>
              ))
            )}
          </div>

          {/* Info */}
          {variables.length > 0 && (
            <div className="p-2 border-t border-slate-700 text-xs text-gray-500">
              <p>Total: {variables.length} variable(s)</p>
            </div>
          )}
        </>
      )}
    </div>
  )
}
