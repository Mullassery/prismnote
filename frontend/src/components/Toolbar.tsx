import { Download, Upload, Save, Moon, Sun, Sparkles, RotateCcw } from 'lucide-react'
import { useNotebookStore } from '../hooks/useNotebook'
import AISettings from './AISettings'
import { restartKernel } from '../api/kernel'
import { useState } from 'react'

export default function Toolbar() {
  const [isDark, setIsDark] = useState(true)
  const [showAISettings, setShowAISettings] = useState(false)
  const { currentNotebook, saveNotebook } = useNotebookStore()

  const handleExport = () => {
    if (!currentNotebook) return
    const data = JSON.stringify(currentNotebook, null, 2)
    const blob = new Blob([data], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `${currentNotebook.name}.ipynb`
    a.click()
  }

  return (
    <div className="h-14 bg-slate-800 border-b border-slate-700 flex items-center justify-between px-4">
      <div className="flex items-center gap-2">
        <h2 className="text-lg font-semibold text-white">{currentNotebook?.name}</h2>
      </div>

      <div className="flex items-center gap-3">
        <button
          onClick={saveNotebook}
          className="p-2 hover:bg-slate-700 rounded transition text-gray-400 hover:text-white"
          title="Save notebook"
        >
          <Save size={18} />
        </button>

        <button
          onClick={handleExport}
          className="p-2 hover:bg-slate-700 rounded transition text-gray-400 hover:text-white"
          title="Export as .ipynb"
        >
          <Download size={18} />
        </button>

        <button
          className="p-2 hover:bg-slate-700 rounded transition text-gray-400 hover:text-white"
          title="Import notebook"
        >
          <Upload size={18} />
        </button>

        <button
          onClick={async () => {
            if (confirm('Restart the kernel? All variables will be cleared.')) {
              await restartKernel()
            }
          }}
          className="p-2 hover:bg-slate-700 rounded transition text-gray-400 hover:text-white"
          title="Restart kernel (clear all variables)"
        >
          <RotateCcw size={18} />
        </button>

        <div className="h-6 w-px bg-slate-700"></div>

        <button
          onClick={() => setShowAISettings(true)}
          className="p-2 hover:bg-slate-700 rounded transition text-gray-400 hover:text-white"
          title="AI Settings"
        >
          <Sparkles size={18} />
        </button>

        <button
          onClick={() => setIsDark(!isDark)}
          className="p-2 hover:bg-slate-700 rounded transition text-gray-400 hover:text-white"
        >
          {isDark ? <Sun size={18} /> : <Moon size={18} />}
        </button>
      </div>

      <AISettings isOpen={showAISettings} onClose={() => setShowAISettings(false)} />
    </div>
  )
}
